#include <assert.h>
#include <errno.h>
#include <setjmp.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>

#include "guard.h"

#define GD_PAGE_BITS              14ULL
#define GD_PAGE_SIZE              (1ULL << GD_PAGE_BITS)    // 16 KB
#define GD_PAGE_MASK              (GD_PAGE_SIZE - 1)
#define GD_PAGE_ROUND_DOWN(foo)   (foo & (~GD_PAGE_MASK))

static uintptr_t         guard_p = 0;
static const uintptr_t  *stack_pp = NULL;
static const uintptr_t  *alloc_pp = NULL;
static BufListNode      *buffer_list = NULL;
static struct sigaction  prev_sa;

static int32_t
_prot_page(void *address, int prot)
{
  if (mprotect(address, GD_PAGE_SIZE, prot)) {
    fprintf(stderr, "guard: prot: mprotect error %d\r\n", errno);
    fprintf(stderr, "%s\r\n", strerror(errno));
    return guard_mprotect | errno;
  }

  return 0;
}

static int32_t
_mark_page(void *address)
{
  return _prot_page(address, PROT_NONE);
}

static int32_t
_unmark_page(void *address)
{
  return _prot_page(address, PROT_READ | PROT_WRITE);
}

// Center the guard page.
// XX: could be a false positive if the new frame results in exact same guard page
//     solution: we only re-center from the signal handler
static int32_t
_focus_guard()
{
  uintptr_t stack_p = *stack_pp;
  uintptr_t alloc_p = *alloc_pp;
  uintptr_t old_guard_p = guard_p;
  uintptr_t new_guard_p;
  int32_t   err = 0;

  fprintf(stderr, "guard: focus: stack pointer at %p\r\n", (void *)stack_p);
  fprintf(stderr, "guard: focus: alloc pointer at %p\r\n", (void *)alloc_p);

  if (stack_p == 0 || alloc_p == 0) {
    fprintf(stderr, "guard: focus: stack or alloc pointer is null\r\n");
    return guard_null;
  } else if (stack_p == alloc_p) {
    fprintf(stderr, "guard: focus: stack and alloc pointers equal\r\n");
    return guard_oom;
  }

  fprintf(stderr, "guard: focus: old guard = %p\r\n", (void *)old_guard_p);

  // Compute new guard page
  // XX:  Should we also check for new_guard_p < min(stack_p, alloc_p)?
  new_guard_p = GD_PAGE_ROUND_DOWN((stack_p + alloc_p) / 2);
  fprintf(stderr, "guard: focus: new guard = %p\r\n", (void *)new_guard_p);
  if (new_guard_p == old_guard_p) {
    fprintf(stderr, "guard: focus: OOM\r\n");
    return guard_oom;
  }

  // Mark new guard page
  if ((err = _mark_page((void *)new_guard_p))) {
    fprintf(stderr, "guard: focus: mark error\r\n");
    return err;
  }

  // Update guard page tracker
  fprintf(stderr, "guard: focus: installed guard page at %p\r\n", (void *)new_guard_p);
  guard_p = new_guard_p;

  // Unmark the old guard page (if there is one)
  if (old_guard_p) {
    if ((err = _unmark_page((void *)old_guard_p))) {
      fprintf(stderr, "guard: focus: unmark error\r\n");
      return err;
    }
  }

  return 0;
}

static void
_signal_handler(int sig, siginfo_t *si, void *unused)
{
  uintptr_t sig_addr;
  int32_t   err = 0;

  assert(guard_p);

  fprintf(stderr, "guard: sig_handle: %d received\r\n", sig);

  if (sig != SIGSEGV) {
    fprintf(stderr, "guard: sig_handle: invalid signal\r\n");
    // XX: do we even want to jump? if this is fatal error, maybe just die now
    siglongjmp(buffer_list->buffer, guard_signal);
  }

  sig_addr = (uintptr_t)si->si_addr;
  fprintf(stderr, "guard: SIGSEGV address = %p\r\n", (void *)sig_addr);

  if (sig_addr >= guard_p &&
      sig_addr <  guard_p + GD_PAGE_SIZE)
  {
    fprintf(stderr, "guard: hit: %p\r\n", si->si_addr);
    err = _focus_guard();
    if (err) {
      fprintf(stderr, "guard: sig_handle: focus error\r\n");
      siglongjmp(buffer_list->buffer, err);
    }
  }
  else {
    fprintf(stderr, "guard: page at %p miss\r\n", (void *)guard_p);

    if (prev_sa.sa_sigaction != NULL) {
      prev_sa.sa_sigaction(sig, si, unused);
    } else if (prev_sa.sa_handler != NULL) {
      prev_sa.sa_handler(sig);
    } else {
      // There should always be a default SIGSEGV handler
      assert(0);
    }
  }
}

static int32_t
_register_handler()
{
  struct sigaction sa;

  // Flag to use sa_sigaction
  sa.sa_flags = SA_SIGINFO;
  // Must use sa_sigaction; sa-handler takes signal handler as its only argument
  sa.sa_sigaction = _signal_handler;
  // Set mask of signals to ignore while running signal handler
  // TODO:  By default the signal that triggered the signal handler is automatically added to the
  //        mask while it's being handled, so unless we plan to add more signals to this then I
  //        don't think it's necessary.
  // sigemptyset(&sa.sa_mask);
  // sigaddset(&(sa.sa_mask), SIGSEGV);

  // XX: should assert that prev_sa is uninitialized, but it's not a pointer so non-trivial
  if (sigaction(SIGSEGV, &sa, &prev_sa)) {
    fprintf(stderr, "guard: register: sigaction error\r\n");
    fprintf(stderr, "%s\r\n", strerror(errno));
    return guard_sigaction | errno;
  }

  return 0;
}

int32_t
guard(
  callback f,
  void *closure,
  const uintptr_t *const s_pp,
  const uintptr_t *const a_pp,
  void ** ret
) {
  BufListNode  *new_buffer;
  int32_t       err = 0;
  int32_t       td_err = 0;

  fprintf(stderr, "guard: setup: stack pointer at %p\r\n", (void *)(*s_pp));
  fprintf(stderr, "guard: setup: alloc pointer at %p\r\n", (void *)(*a_pp));

  if (guard_p == 0) {
    assert(buffer_list == NULL);

    stack_pp = s_pp;
    alloc_pp = a_pp;

    // Initialize the guard page
    if ((err = _focus_guard())) {
      fprintf(stderr, "guard: setup _focus_guard error\r\n");
      goto exit;
    }

    // Register guard page signal handler
    if ((err = _register_handler())) {
      fprintf(stderr, "guard: setup _register_handler error\r\n");
      goto clean;
    }
  } else {
    assert(buffer_list != NULL);
  }

  // Setup new longjmp buffer
  new_buffer = (BufListNode *)malloc(sizeof(BufListNode));
  if (new_buffer == NULL) {
    fprintf(stderr, "guard: malloc error\r\n");
    fprintf(stderr, "%s\r\n", strerror(errno));
    err = guard_malloc | errno;
    goto skip;
  }
  new_buffer->next = buffer_list;
  buffer_list = new_buffer;

  // Run given closure
  fprintf(stderr, "guard: run\r\n");
  if (!(err = sigsetjmp(buffer_list->buffer, 1))) {
    *ret = f(closure);
  }

  // Restore previous longjmp buffer
  buffer_list = buffer_list->next;
  free((void *)new_buffer);

skip:
  // If no more guarded closures, then...
  if (buffer_list == NULL) {
    // Remove new signal handler
    if (sigaction(SIGSEGV, &prev_sa, NULL)) {
      fprintf(stderr, "guard: sigaction error\r\n");
      fprintf(stderr, "%s\r\n", strerror(errno));
      td_err = guard_sigaction | errno;

      if (!err) {
        err = td_err;
      }
    }

clean:
    // Unmark guard page
    assert(guard_p != 0);
    td_err = _unmark_page((void *)guard_p);
    if (td_err) {
      fprintf(stderr, "guard: unmark error\r\n");
      fprintf(stderr, "%s\r\n", strerror(errno));
      if (!err) {
        err = td_err;
      }
    }
    guard_p = 0;
  }

exit:
  fprintf(stderr, "guard: return\r\n");
  return err;
}