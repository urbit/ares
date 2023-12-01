use ares_macros::tas;
use either::Either::{Left, Right};
use std::mem::size_of;
use std::ptr::copy_nonoverlapping;
use std::result::Result;

use crate::hamt::Hamt;
use crate::interpreter::{mean_pop, mean_push, slow_pop, slow_push, Context, Error};
use crate::jets::util::slot;
use crate::mem::{NockStack, Preserve};
use crate::noun::{Noun, D, T};
use crate::trace::TraceStack;

// XX typedef for register
#[derive(Copy, Clone)]
pub struct PileMem {
    long: Noun,
    want: Noun,
    wish: Noun,
    sire: usize,
    will: Hamt<Noun>,
    sans: usize,
}

#[derive(Copy, Clone)]
pub struct Pile(*const PileMem);
impl Preserve for Pile {
    unsafe fn preserve(&mut self, stack: &mut NockStack) {
        if stack.is_in_frame(self.0) {
            let mut pile_mem = *(self.0);
            pile_mem.long.preserve(stack);
            pile_mem.want.preserve(stack);
            pile_mem.wish.preserve(stack);
            pile_mem.will.preserve(stack);
            let dest_mem: *mut PileMem = stack.struct_alloc_in_previous_frame(1);
            copy_nonoverlapping(self.0, dest_mem, 1);
        }
    }

    unsafe fn assert_in_stack(&self, stack: &NockStack) {
        stack.assert_struct_is_in(self.0, 1);
        (*(self.0)).long.assert_in_stack(stack);
        (*(self.0)).want.assert_in_stack(stack);
        (*(self.0)).wish.assert_in_stack(stack);
        (*(self.0)).will.assert_in_stack(stack);
    }
}

impl Pile {
    fn from_noun(stack: &mut NockStack, p: Noun) -> Result<Pile, Error> {
        unsafe {
            let mem: *mut PileMem = stack.struct_alloc(1);
            *mem = PileMem {
                long: slot(p, 2)?,
                want: slot(p, 6)?,
                wish: slot(p, 14)?,
                sire: slot(p, 30)?.as_direct()?.data() as usize,
                will: util::part_will(stack, slot(p, 62)?)?,
                sans: slot(p, 63)?.as_direct()?.data() as usize,
            };
            Ok(Pile(mem))
        }
    }
}

/// First peeks or pokes the codegen core (line) to get codegen for the
/// subject and formula, then parses the successful results into a
/// (bell, hill) tuple.
fn cg_pull_peek(
    context: &mut Context,
    subject: Noun,
    formula: Noun,
) -> Result<(Noun, Hamt<Pile>), Error> {
    // +peek or +poke dance
    context.line.ok_or(Error::Deterministic(D(0)))?;
    let pek = util::peek(context, subject, formula)?;
    if unsafe { pek.raw_equals(D(0)) } {
        let comp = util::comp(context, subject, formula);
        let line = util::poke(context, comp).expect("poke failed");
        context.line = Some(line);
        let good_peek = util::peek(context, subject, formula)?;
        context.peek = Some(util::part_peek(&mut context.stack, good_peek)?);
    } else {
        context.peek = Some(util::part_peek(&mut context.stack, pek)?);
    }

    // parse the peek's non-empty results and return them
    if context.peek.is_none() {
        return Err(Error::Deterministic(D(0)));
    }
    let bell = context.peek.unwrap().0;
    let hill = context.peek.unwrap().1;
    Ok((bell, hill))
}

/// Uses the `(bell, hill)` tuple returned from `cg_pull_peek()` to lookup the first
/// `pile` in the `hill` map.
/// XX percolate the changes to this through the code
fn cg_pull_pile(context: &mut Context, subject: Noun, formula: Noun) -> Result<Pile, Error> {
    let (mut bell, hill) = cg_pull_peek(context, subject, formula).unwrap();
    let pile = hill
        .lookup(&mut context.stack, &mut bell)
        .ok_or(Error::Deterministic(D(0)))?;
    Ok(pile)
}

struct Frame {
    mean: Noun,
    traz: *mut *const TraceStack,
    slow: Noun,
    pile: Pile,
    dest: usize,
    cont: Noun,
    pois_sz: usize, // length of poison vector
                    // poison: Vec<u64>,     // variable size
                    // registers: Vec<Noun>, // variable size
}

/// Fetches or creates codegen code for the subject and formula, then
/// naively interprets it.
pub fn cg_interpret(context: &mut Context, subject: Noun, formula: Noun) -> Result<Noun, Error> {
    let mut virtual_frame: *mut Frame = std::ptr::null::<Frame>() as *mut Frame;
    let base_frame = context.stack.get_frame_pointer();

    // Setup stack for codegen interpretation.
    // Stack frame layout: [mean trace slow pile dest cont poison registers]
    // XX update local_noun_pointer() calls with correct constants
    {
        let pile = cg_pull_pile(context, subject, formula)?;
        let sans = unsafe { (*(pile.0)).sans };
        let pois_sz = (sans / 64) + if (sans % 64) == 0 { 0 } else { 1 };
        context
            .stack
            .frame_push(size_of::<Frame>() + pois_sz + sans);
        virtual_frame = context.stack.get_frame_pointer() as *mut Frame;
        unsafe {
            (*(virtual_frame)).mean = D(0);
            (*(virtual_frame)).traz = std::ptr::null::<TraceStack>() as *mut *const TraceStack;
            (*(virtual_frame)).slow = D(0);

            (*(virtual_frame)).pile = pile;

            (*(virtual_frame)).dest = 0;
            (*(virtual_frame)).cont = D(0);
            (*(virtual_frame)).pois_sz = pois_sz;
        }
    }

    // Load the initial subject to the sire register.
    {
        let sire = unsafe { (*(*virtual_frame).pile.0).sire };
        register_set(virtual_frame, sire, subject);
    }

    // Get the blob, body, and bend nouns from our pile.
    let mut body: Noun;
    let mut bend: Noun;
    {
        let will = unsafe { (*(*virtual_frame).pile.0).will };
        let blob = will
            .lookup(&mut context.stack, &mut unsafe {
                (*(*virtual_frame).pile.0).wish
            })
            .ok_or(Error::Deterministic(D(0)))?;
        body = slot(blob, 2)?;
        bend = slot(blob, 3)?;
    }

    loop {
        if !unsafe { body.raw_equals(D(0)) } {
            let pole = slot(body, 2)?;
            body = slot(body, 3)?;
            match slot(pole, 2)?.as_direct()?.data() {
                tas!(b"imm") => {
                    let local = slot(pole, 7)?.as_direct()?.data() as usize;
                    let value = slot(pole, 6)?;
                    register_set(virtual_frame, local, value);
                }
                tas!(b"mov") => {
                    let src = slot(pole, 6)?.as_direct()?.data() as usize;
                    let dst = slot(pole, 7)?.as_direct()?.data() as usize;
                    let value = register_get(virtual_frame, src);
                    register_set(virtual_frame, dst, value);
                }
                tas!(b"inc") => {}
                tas!(b"con") => {
                    let h = slot(pole, 6)?.as_direct()?.data() as usize;
                    let t = slot(pole, 14)?.as_direct()?.data() as usize;
                    let d = slot(pole, 15)?.as_direct()?.data() as usize;
                    let h_value = register_get(virtual_frame, h);
                    let t_value = register_get(virtual_frame, t);
                    let value = T(&mut context.stack, &[h_value, t_value]);
                    register_set(virtual_frame, d, value);
                }
                tas!(b"cop") => {}
                tas!(b"lop") => {}
                tas!(b"coc") => {}
                tas!(b"hed") => {
                    let s = slot(pole, 6)?.as_direct()?.data() as usize;
                    let d = slot(pole, 7)?.as_direct()?.data() as usize;
                    let s_value = register_get(virtual_frame, s);
                    match s_value.as_either_atom_cell() {
                        Left(_atom) => {
                            poison_set(virtual_frame, s);
                        }
                        Right(cell) => {
                            register_set(virtual_frame, d, cell.head());
                        }
                    };
                }
                tas!(b"tal") => {
                    let s = slot(pole, 6)?.as_direct()?.data() as usize;
                    let d = slot(pole, 7)?.as_direct()?.data() as usize;
                    let s_value = register_get(virtual_frame, s);
                    match s_value.as_either_atom_cell() {
                        Left(_atom) => {
                            poison_set(virtual_frame, s);
                        }
                        Right(cell) => {
                            register_set(virtual_frame, d, cell.tail());
                        }
                    };
                }
                tas!(b"hci") => {
                    let s = slot(pole, 6)?.as_direct()?.data() as usize;
                    let d = slot(pole, 7)?.as_direct()?.data() as usize;
                    let s_value = register_get(virtual_frame, s);
                    match s_value.as_either_atom_cell() {
                        Left(_atom) => {
                            // XX crash
                        }
                        Right(cell) => {
                            register_set(virtual_frame, d, cell.head());
                        }
                    };
                }
                tas!(b"tci") => {
                    let s = slot(pole, 6)?.as_direct()?.data() as usize;
                    let d = slot(pole, 7)?.as_direct()?.data() as usize;
                    let s_value = register_get(virtual_frame, s);
                    match s_value.as_either_atom_cell() {
                        Left(_atom) => {
                            // XX crash
                        }
                        Right(cell) => {
                            register_set(virtual_frame, d, cell.tail());
                        }
                    };
                }
                tas!(b"men") => {
                    let s = slot(pole, 7)?.as_direct()?.data() as usize;
                    let s_value = register_get(virtual_frame, s);
                    mean_push(&mut context.stack, s_value);
                }
                tas!(b"man") => {
                    mean_pop(&mut context.stack);
                }
                tas!(b"slo") => {
                    let s = slot(pole, 3)?.as_direct()?.data() as usize;
                    let s_value = register_get(virtual_frame, s);
                    slow_push(&mut context.stack, s_value);
                }
                tas!(b"sld") => {
                    slow_pop(&mut context.stack);
                }
                tas!(b"hit") => {
                    let s = slot(pole, 3)?.as_direct()?.data() as usize;
                    let _s_value = register_get(virtual_frame, s);
                    // XX increment a profiling hit counter labeled with the noun in s
                }
                tas!(b"slg") => {
                    let s = slot(pole, 3)?.as_direct()?.data() as usize;
                    let clue = register_get(virtual_frame, s);
                    if let Ok(slog_cell) = clue.as_cell() {
                        if let Ok(pri_direct) = slog_cell.head().as_direct() {
                            let tank = slog_cell.tail();
                            context
                                .newt
                                .slog(&mut context.stack, pri_direct.data(), tank);
                        };
                    };
                }
                tas!(b"mew") => {
                    let k = slot(pole, 6)?.as_direct()?.data() as usize;
                    let u = slot(pole, 14)?.as_direct()?.data() as usize;
                    let f = slot(pole, 30)?.as_direct()?.data() as usize;
                    let r = slot(pole, 31)?.as_direct()?.data() as usize;

                    let k_value = register_get(virtual_frame, k);
                    let u_value = register_get(virtual_frame, u);
                    let f_value = register_get(virtual_frame, f);
                    let r_value = register_get(virtual_frame, r);
                    let mut key = T(&mut context.stack, &[k_value, u_value, f_value]);

                    context.cache = context.cache.insert(&mut context.stack, &mut key, r_value);
                }
                tas!(b"tim") => {
                    // XX push a timer onto the stack and start it
                }
                tas!(b"tom") => {
                    // XX pop a timer from the stack, stop it, and print elapsed
                }
                tas!(b"mem") => {
                    // XX print memory usage
                }
                tas!(b"pol") => {
                    let s = slot(pole, 6)?.as_direct()?.data() as usize;
                    let d = slot(pole, 7)?.as_direct()?.data() as usize;
                    if poison_get(virtual_frame, s) {
                        poison_set(virtual_frame, d);
                    }
                }
                tas!(b"poi") => {
                    let d = slot(pole, 3)?.as_direct()?.data() as usize;
                    poison_set(virtual_frame, d);
                }
                tas!(b"ipb") => {
                    let mut s = slot(pole, 3)?;
                    loop {
                        if unsafe { s.raw_equals(D(0)) } {
                            break;
                        } else {
                            let i = s.as_cell()?.head().as_direct()?.data() as usize;
                            if poison_get(virtual_frame, i) {
                                // XX crash
                            } else {
                                s = s.as_cell()?.tail();
                            }
                        }
                    }
                }
                _ => {
                    panic!("invalid pole instruction")
                }
            }
        } else {
            match slot(bend, 2)?.as_direct()?.data() {
                tas!(b"clq") => {
                    let s = slot(bend, 6)?.as_direct()?.data() as usize;
                    let s_value = register_get(virtual_frame, s);
                    match s_value.as_either_atom_cell() {
                        Left(_atom) => {
                            let mut o = slot(bend, 15)?;
                            let will = unsafe { (*(*virtual_frame).pile.0).will };
                            let blob = will
                                .lookup(&mut context.stack, &mut o)
                                .ok_or(Error::Deterministic(D(0)))?;
                            body = slot(blob, 2)?;
                            bend = slot(blob, 3)?;
                            continue;
                        }
                        Right(_cell) => {
                            let mut z = slot(bend, 14)?;
                            let will = unsafe { (*(*virtual_frame).pile.0).will };
                            let blob = will
                                .lookup(&mut context.stack, &mut z)
                                .ok_or(Error::Deterministic(D(0)))?;
                            body = slot(blob, 2)?;
                            bend = slot(blob, 3)?;
                            continue;
                        }
                    };
                }
                tas!(b"eqq") => {
                    let l = slot(bend, 6)?.as_direct()?.data() as usize;
                    let r = slot(bend, 14)?.as_direct()?.data() as usize;
                    let l_value = register_get(virtual_frame, l);
                    let r_value = register_get(virtual_frame, r);
                    if unsafe { l_value.raw_equals(r_value) } {
                        let mut z = slot(bend, 30)?;
                        let will = unsafe { (*(*virtual_frame).pile.0).will };
                        let blob = will
                            .lookup(&mut context.stack, &mut z)
                            .ok_or(Error::Deterministic(D(0)))?;
                        body = slot(blob, 2)?;
                        bend = slot(blob, 3)?;
                        continue;
                    } else {
                        let mut o = slot(bend, 31)?;
                        let will = unsafe { (*(*virtual_frame).pile.0).will };
                        let blob = will
                            .lookup(&mut context.stack, &mut o)
                            .ok_or(Error::Deterministic(D(0)))?;
                        body = slot(blob, 2)?;
                        bend = slot(blob, 3)?;
                        continue;
                    }
                }
                tas!(b"brn") => {
                    let s = slot(bend, 6)?.as_direct()?.data() as usize;
                    let s_value = register_get(virtual_frame, s);
                    if unsafe { s_value.raw_equals(D(0)) } {
                        let mut z = slot(bend, 14)?;
                        let will = unsafe { (*(*virtual_frame).pile.0).will };
                        let blob = will
                            .lookup(&mut context.stack, &mut z)
                            .ok_or(Error::Deterministic(D(0)))?;
                        body = slot(blob, 2)?;
                        bend = slot(blob, 3)?;
                        continue;
                    } else if unsafe { s_value.raw_equals(D(1)) } {
                        let mut o = slot(bend, 15)?;
                        let will = unsafe { (*(*virtual_frame).pile.0).will };
                        let blob = will
                            .lookup(&mut context.stack, &mut o)
                            .ok_or(Error::Deterministic(D(0)))?;
                        body = slot(blob, 2)?;
                        bend = slot(blob, 3)?;
                        continue;
                    } else {
                        // XX crash
                    }
                }
                tas!(b"hop") => {
                    let mut t = slot(bend, 3)?;
                    let will = unsafe { (*(*virtual_frame).pile.0).will };
                    let blob = will
                        .lookup(&mut context.stack, &mut t)
                        .ok_or(Error::Deterministic(D(0)))?;
                    body = slot(blob, 2)?;
                    bend = slot(blob, 3)?;
                    continue;
                }
                // tas!(b"hip") => {
                //     // XX set comefrom label to c and goto t
                // },
                tas!(b"lnk") => {
                    // evaluate f against u and put the result in d, then goto t
                    let u = slot(bend, 6)?.as_direct()?.data() as usize;
                    let f = slot(bend, 14)?.as_direct()?.data() as usize;
                    let d = slot(bend, 30)?.as_direct()?.data() as usize;
                    let t = slot(bend, 31)?;

                    let subject = register_get(virtual_frame, u);
                    let formula = register_get(virtual_frame, f);

                    unsafe {
                        (*virtual_frame).dest = d;
                        (*virtual_frame).cont = t;
                    }

                    {
                        let pile = cg_pull_pile(context, subject, formula)?;
                        let sans = unsafe { (*(pile.0)).sans };
                        let pois_sz = (sans / 64) + if (sans % 64) == 0 { 0 } else { 1 };
                        context
                            .stack
                            .frame_push(size_of::<Frame>() + pois_sz + sans);
                        virtual_frame = context.stack.get_frame_pointer() as *mut Frame;
                        unsafe {
                            (*(virtual_frame)).mean = D(0);
                            (*(virtual_frame)).traz =
                                std::ptr::null::<TraceStack>() as *mut *const TraceStack;
                            (*(virtual_frame)).slow = D(0);

                            (*(virtual_frame)).pile = pile;

                            (*(virtual_frame)).dest = 0;
                            (*(virtual_frame)).cont = D(0);
                            (*(virtual_frame)).pois_sz = pois_sz;
                        }
                    }

                    {
                        let will = unsafe { (*(*virtual_frame).pile.0).will };
                        let blob = will
                            .lookup(&mut context.stack, &mut unsafe {
                                (*(*virtual_frame).pile.0).wish
                            })
                            .ok_or(Error::Deterministic(D(0)))?;
                        body = slot(blob, 2)?;
                        bend = slot(blob, 3)?;
                    }
                    continue;
                }
                tas!(b"cal") => {
                    // call the arm a with subject in registers u, poisons in b,
                    // result in d, and then goto t
                }
                tas!(b"caf") => {
                    // like call but with fast label
                }
                tas!(b"lnt") => {
                    // evaluate f against u in tail position
                }
                tas!(b"jmp") => {
                    // call the arm a with subject in registers u, poisons in b, in
                    // tail position
                }
                tas!(b"jmf") => {
                    // like jmp but with fast label
                }
                tas!(b"spy") => {
                    // scry with ref in e and path in p, put result in d, goto t
                }
                tas!(b"mer") => {
                    // check if triple [k u f] is in cache, put result in d if so
                    // and goto i, else goto m
                }
                tas!(b"don") => {
                    let s = slot(bend, 6)?.as_direct()?.data() as usize;
                    let mut s_value = register_get(virtual_frame, s);

                    unsafe {
                        let stack = &mut context.stack;
                        // XX debug assertions

                        stack.preserve(&mut context.cache);
                        stack.preserve(&mut context.cold);
                        stack.preserve(&mut context.warm);
                        stack.preserve(&mut s_value);
                        stack.frame_pop();

                        let fp = context.stack.get_frame_pointer();
                        if fp == base_frame {
                            break Ok(s_value);
                        }
                        virtual_frame = fp as *mut Frame;

                        register_set(virtual_frame, (*virtual_frame).dest, s_value);

                        let will = (*(*virtual_frame).pile.0).will;
                        let blob = will
                            .lookup(&mut context.stack, &mut (*virtual_frame).cont)
                            .ok_or(Error::Deterministic(D(0)))?;
                        body = slot(blob, 2)?;
                        bend = slot(blob, 3)?;
                    }
                    continue;
                }
                tas!(b"bom") => {
                    // crash
                }
                _ => {
                    panic!("invalid bend instruction");
                }
            }
        }
    }
}

// XX implement a generic local_set that all the more specific
// getters and setters call
fn register_set(frame: *mut Frame, local: usize, value: Noun) {
    unsafe {
        let pois_sz = (*(frame)).pois_sz;
        let reg_ptr = (frame as *mut Noun).add(size_of::<Frame>() + pois_sz + local);
        *reg_ptr = value;
    }
}

fn register_get(frame: *const Frame, local: usize) -> Noun {
    unsafe {
        let pois_sz = (*(frame)).pois_sz;
        let reg_ptr = (frame as *mut Noun).add(size_of::<Frame>() + pois_sz + local);
        *reg_ptr
    }
}

fn poison_set(frame: *mut Frame, local: usize) {
    let index = local / 64;
    let offset = local % 64;
    let bitmap = unsafe { (frame as *mut u64).add(size_of::<Frame>() + index) };
    unsafe { *bitmap |= 1 << offset };
}

fn poison_get(frame: *const Frame, local: usize) -> bool {
    let index = local / 64;
    let offset = local % 64;
    let bitmap = unsafe { *(frame as *const u64).add(size_of::<Frame>() + index) };
    bitmap & (1 << offset) != 0
}

pub mod util {
    use ares_macros::tas;

    use crate::{
        hamt::Hamt,
        interpreter::{Context, Error},
        jets::util::{kick, slam, slot},
        mem::NockStack,
        noun::{Noun, D, T},
    };

    use super::Pile;

    pub type NounResult = Result<Noun, Error>;

    pub fn peek(context: &mut Context, subject: Noun, formula: Noun) -> NounResult {
        // +peek slot in line core is 4
        let line = context.line.ok_or(Error::Deterministic(D(0)))?;
        let pek = kick(context, line, D(4))?;
        let sam = T(&mut context.stack, &[subject, formula]);
        slam(context, pek, sam).map_err(|_| Error::Deterministic(D(0)))
    }

    pub fn poke(context: &mut Context, gist: Noun) -> NounResult {
        // +poke slot in line core is 86
        let line = context.line.ok_or(Error::Deterministic(D(0)))?;
        let pok = kick(context, line, D(86))?;
        let sam = T(&mut context.stack, &[gist]);
        slam(context, pok, sam).map_err(|_| Error::Deterministic(D(0)))
    }

    pub fn tap(stack: &mut NockStack, map: Noun) -> NounResult {
        tap_in(stack, slot(map, 30)?, D(0))
    }

    fn tap_in(stack: &mut NockStack, a: Noun, mut b: Noun) -> NounResult {
        unsafe {
            stack.frame_push(0);
            *(stack.push::<Noun>()) = a;
            loop {
                if stack.stack_is_empty() {
                    break;
                }
                let tree = *(stack.top::<Noun>());
                stack.pop::<Noun>();
                if tree.raw_equals(D(0)) {
                    continue;
                }
                // XX needs to pop frame even if it fails
                let node = slot(tree, 2)?;
                let lr = slot(tree, 3)?;
                let l = slot(lr, 2)?;
                let r = slot(lr, 3)?;
                b = T(stack, &[node, b]);
                *(stack.push::<Noun>()) = r;
                *(stack.push::<Noun>()) = l;
            }
            stack.preserve(&mut b);
            stack.frame_pop();
            Ok(b)
        }
    }

    pub fn part_hill(stack: &mut NockStack, hill: Noun) -> Result<Hamt<Pile>, Error> {
        let mut kvs = tap(stack, hill)?;
        let mut hamt = Hamt::new();
        while !unsafe { kvs.raw_equals(D(0)) } {
            let c = kvs.as_cell()?;
            let kv = c.head();
            let mut bell = slot(kv, 2)?;
            let pile = Pile::from_noun(stack, slot(kv, 3)?)?;
            hamt = hamt.insert(stack, &mut bell, pile);
            kvs = c.tail();
        }
        Ok(hamt)
    }

    pub fn part_will(stack: &mut NockStack, will: Noun) -> Result<Hamt<Noun>, Error> {
        let mut kvs = tap(stack, will)?;
        let mut hamt = Hamt::new();
        while !unsafe { kvs.raw_equals(D(0)) } {
            let c = kvs.as_cell()?;
            let kv = c.head();
            let mut bile = slot(kv, 2)?;
            let blob = slot(kv, 3)?;
            hamt = hamt.insert(stack, &mut bile, blob);
            kvs = c.tail();
        }
        Ok(hamt)
    }

    pub fn part_peek(stack: &mut NockStack, peek: Noun) -> Result<(Noun, Hamt<Pile>), Error> {
        let bell = slot(peek, 6)?;
        let hall = part_hill(stack, slot(peek, 7)?)?;
        Ok((bell, hall))
    }

    pub fn comp(context: &mut Context, s: Noun, f: Noun) -> Noun {
        T(&mut context.stack, &[D(tas!(b"comp")), D(0), s, f])
    }
}
