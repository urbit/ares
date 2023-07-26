use self::NockWork::*;
use crate::hamt::Hamt;
use crate::jets;
use crate::mem::unifying_equality;
use crate::mem::NockStack;
use crate::newt::Newt;
use crate::noun::{Atom, Cell, DirectAtom, IndirectAtom, Noun};
use ares_macros::tas;
use assert_no_alloc::assert_no_alloc;
use bitvec::prelude::{BitSlice, Lsb0};
use either::Either::*;
use num_traits::cast::{FromPrimitive, ToPrimitive};

crate::gdb!();

#[derive(Copy, Clone, FromPrimitive, ToPrimitive, Debug)]
#[repr(u64)]
enum NockWork {
    Done,
    NockCellComputeHead,
    NockCellComputeTail,
    NockCellCons,
    Nock0Axis,
    Nock1Constant,
    Nock2ComputeSubject,
    Nock2ComputeFormula,
    Nock2ComputeResult,
    Nock2RestoreSubject,
    Nock3ComputeChild,
    Nock3ComputeType,
    Nock4ComputeChild,
    Nock4Increment,
    Nock5ComputeLeftChild,
    Nock5ComputeRightChild,
    Nock5TestEquals,
    Nock6ComputeTest,
    Nock6ComputeBranch,
    Nock6Done,
    Nock7ComputeSubject,
    Nock7ComputeResult,
    Nock7RestoreSubject,
    Nock8ComputeSubject,
    Nock8ComputeResult,
    Nock8RestoreSubject,
    Nock9ComputeCore,
    Nock9ComputeResult,
    Nock9RestoreSubject,
    Nock10ComputeTree,
    Nock10ComputePatch,
    Nock10Edit,
    Nock11ComputeHint,
    Nock11ComputeResult,
    Nock11Done,
}

fn work_to_noun(work: NockWork) -> Noun {
    unsafe {
        DirectAtom::new_unchecked(work.to_u64().expect("IMPOSSIBLE: work does not fit in u64"))
            .as_atom()
            .as_noun()
    }
}

fn noun_to_work(noun: Noun) -> NockWork {
    if let Left(direct) = noun.as_either_direct_allocated() {
        NockWork::from_u64(direct.data()).expect("Invalid work")
    } else {
        panic!("Work should always be a direct atom.")
    }
}

/** Interpret nock */
pub fn interpret(
    stack: &mut NockStack,
    newt: &mut Option<&mut Newt>, // For printing slogs; if None, print to stdout
    mut subject: Noun,
    formula: Noun,
) -> Noun {
    let mut res = unsafe { DirectAtom::new_unchecked(0).as_atom().as_noun() };
    stack.push(0);
    let mut cache = Hamt::<Noun>::new();
    unsafe {
        *(stack.stack_push()) = work_to_noun(Done);
    }
    push_formula(stack, formula);
    assert_no_alloc(|| unsafe {
        loop {
            let work = noun_to_work(*stack.stack_top());
            stack.stack_pop::<Noun>();
            match work {
                Done => {
                    stack.pre_copy();
                    stack.preserve(&mut cache);
                    stack.preserve(&mut res);
                    stack.pop();
                    break;
                }
                NockCellComputeHead => {
                    let formula = *(stack.stack_top::<Noun>());
                    stack.stack_pop::<Noun>();
                    *(stack.stack_push()) = work_to_noun(NockCellComputeTail);
                    push_formula(stack, formula);
                }
                NockCellComputeTail => {
                    let formula = *(stack.stack_top::<Noun>());
                    stack.stack_pop::<Noun>();
                    *(stack.stack_push()) = res;
                    *(stack.stack_push()) = work_to_noun(NockCellCons);
                    push_formula(stack, formula);
                }
                NockCellCons => {
                    let head = *(stack.stack_top::<Noun>());
                    stack.stack_pop::<Noun>();
                    res = Cell::new(stack, head, res).as_noun();
                }
                Nock0Axis => {
                    if let Ok(atom) = (*(stack.stack_top::<Noun>())).as_atom() {
                        stack.stack_pop::<Noun>();
                        res = slot(subject, atom.as_bitslice());
                    } else {
                        panic!("Axis must be atom");
                    };
                }
                Nock1Constant => {
                    res = *stack.stack_top();
                    stack.stack_pop::<Noun>();
                }
                Nock2ComputeSubject => {
                    *(stack.stack_push()) = work_to_noun(Nock2ComputeFormula);
                    let formula = *stack.local_noun_pointer(1);
                    push_formula(stack, formula);
                }
                Nock2ComputeFormula => {
                    *(stack.stack_push()) = work_to_noun(Nock2ComputeResult);
                    *(stack.local_noun_pointer(1)) = res;
                    let formula = *stack.local_noun_pointer(2);
                    push_formula(stack, formula);
                }
                Nock2ComputeResult => {
                    *(stack.stack_push()) = work_to_noun(Nock2RestoreSubject);
                    *(stack.local_noun_pointer(2)) = subject;
                    subject = *(stack.local_noun_pointer(1));
                    push_formula(stack, res);
                }
                Nock2RestoreSubject => {
                    subject = *(stack.local_noun_pointer(2));
                    stack.pre_copy();
                    stack.preserve(&mut cache);
                    stack.preserve(&mut res);
                    stack.pop();
                }
                Nock3ComputeChild => {
                    let formula = *(stack.stack_top::<Noun>());
                    stack.stack_pop::<Noun>();
                    *(stack.stack_push()) = work_to_noun(Nock3ComputeType);
                    push_formula(stack, formula);
                }
                Nock3ComputeType => {
                    res = if res.is_cell() {
                        DirectAtom::new_unchecked(0).as_atom().as_noun()
                    } else {
                        DirectAtom::new_unchecked(1).as_atom().as_noun()
                    };
                }
                Nock4ComputeChild => {
                    let formula = *(stack.stack_top::<Noun>());
                    stack.stack_pop::<Noun>();
                    *(stack.stack_push()) = work_to_noun(Nock4Increment);
                    push_formula(stack, formula);
                }
                Nock4Increment => {
                    if let Ok(atom) = res.as_atom() {
                        res = inc(stack, atom).as_noun();
                    } else {
                        panic!("Cannot increment (Nock 4) a cell");
                    };
                }
                Nock5ComputeLeftChild => {
                    let formula = *(stack.stack_top::<Noun>());
                    stack.stack_pop::<Noun>();
                    *(stack.stack_push()) = work_to_noun(Nock5ComputeRightChild);
                    push_formula(stack, formula);
                }
                Nock5ComputeRightChild => {
                    let formula = *(stack.stack_top::<Noun>());
                    stack.stack_pop::<Noun>();
                    *(stack.stack_push()) = res;
                    *(stack.stack_push()) = work_to_noun(Nock5TestEquals);
                    push_formula(stack, formula);
                }
                Nock5TestEquals => {
                    let saved_value_ptr = stack.stack_top::<Noun>();
                    res = if unifying_equality(stack, &mut res, saved_value_ptr) {
                        DirectAtom::new_unchecked(0).as_atom().as_noun()
                    } else {
                        DirectAtom::new_unchecked(1).as_atom().as_noun()
                    };
                    stack.stack_pop::<Noun>();
                }
                Nock6ComputeTest => {
                    let formula = *(stack.stack_top::<Noun>());
                    stack.stack_pop::<Noun>();
                    *(stack.stack_push()) = work_to_noun(Nock6ComputeBranch);
                    push_formula(stack, formula);
                }
                Nock6ComputeBranch => {
                    if let Left(direct) = res.as_either_direct_allocated() {
                        if direct.data() == 0 {
                            let formula = *(stack.stack_top::<Noun>());
                            stack.stack_pop::<Noun>();
                            stack.stack_pop::<Noun>();
                            *(stack.stack_push()) = work_to_noun(Nock6Done);
                            push_formula(stack, formula);
                        } else if direct.data() == 1 {
                            stack.stack_pop::<Noun>();
                            let formula = *(stack.stack_top::<Noun>());
                            stack.stack_pop::<Noun>();
                            *(stack.stack_push()) = work_to_noun(Nock6Done);
                            push_formula(stack, formula);
                        } else {
                            panic!("Test branch of Nock 6 must return 0 or 1");
                        };
                    } else {
                        panic!("Test branch of Nock 6 must return a direct atom");
                    }
                }
                Nock6Done => {
                    //TODO remove this
                }
                Nock7ComputeSubject => {
                    let formula = *(stack.stack_top::<Noun>());
                    stack.stack_pop::<Noun>();
                    *(stack.stack_push()) = work_to_noun(Nock7ComputeResult);
                    push_formula(stack, formula);
                }
                Nock7ComputeResult => {
                    let formula = *(stack.stack_top::<Noun>());
                    stack.stack_pop::<Noun>();
                    *(stack.stack_push()) = subject;
                    *(stack.stack_push()) = work_to_noun(Nock7RestoreSubject);
                    subject = res;
                    push_formula(stack, formula);
                }
                Nock7RestoreSubject => {
                    subject = *(stack.stack_top::<Noun>());
                    stack.stack_pop::<Noun>();
                }
                Nock8ComputeSubject => {
                    let formula = *(stack.stack_top::<Noun>());
                    stack.stack_pop::<Noun>();
                    *(stack.stack_push()) = work_to_noun(Nock8ComputeResult);
                    push_formula(stack, formula);
                }
                Nock8ComputeResult => {
                    let formula = *(stack.stack_top::<Noun>());
                    stack.stack_pop::<Noun>();
                    *(stack.stack_push()) = subject;
                    *(stack.stack_push()) = work_to_noun(Nock8RestoreSubject);
                    subject = Cell::new(stack, res, subject).as_noun();
                    push_formula(stack, formula);
                }
                //TODO do I actually need this anymore?
                Nock8RestoreSubject => {
                    subject = *(stack.stack_top::<Noun>());
                    stack.stack_pop::<Noun>();
                }
                Nock9ComputeCore => {
                    *(stack.stack_push()) = work_to_noun(Nock9ComputeResult);
                    let formula = *stack.local_noun_pointer(2);
                    push_formula(stack, formula);
                }
                Nock9ComputeResult => {
                    if let Ok(formula_axis) = (*(stack.local_noun_pointer(1))).as_atom() {
                        *(stack.stack_push()) = work_to_noun(Nock9RestoreSubject);
                        *(stack.local_noun_pointer(2)) = subject;
                        subject = res;
                        push_formula(stack, slot(subject, formula_axis.as_bitslice()));
                    } else {
                        panic!("Axis into core must be atom");
                    }
                }
                Nock9RestoreSubject => {
                    subject = *(stack.local_noun_pointer(2));
                    stack.pre_copy();
                    stack.preserve(&mut cache);
                    stack.preserve(&mut res);
                    stack.pop();
                }
                Nock10ComputeTree => {
                    let formula = *(stack.stack_top::<Noun>());
                    stack.stack_pop::<Noun>();
                    *(stack.stack_push()) = work_to_noun(Nock10ComputePatch);
                    push_formula(stack, formula);
                }
                Nock10ComputePatch => {
                    let formula = *(stack.stack_top::<Noun>());
                    stack.stack_pop::<Noun>();
                    let axis = *(stack.stack_top::<Noun>());
                    stack.stack_pop::<Noun>();
                    *(stack.stack_push::<Noun>()) = res;
                    *(stack.stack_push::<Noun>()) = axis;
                    *(stack.stack_push::<Noun>()) = work_to_noun(Nock10Edit);
                    push_formula(stack, formula);
                }
                Nock10Edit => {
                    if let Ok(edit_axis) = (*stack.stack_top::<Noun>()).as_atom() {
                        stack.stack_pop::<Noun>();
                        let tree = *stack.stack_top::<Noun>();
                        stack.stack_pop::<Noun>();
                        res = edit(stack, edit_axis.as_bitslice(), res, tree);
                    } else {
                        panic!("Axis into tree must be atom");
                    }
                }
                Nock11ComputeHint => {
                    let hint = *stack.local_noun_pointer(1);
                    if let Ok(hint_cell) = hint.as_cell() {
                        let formula = *stack.local_noun_pointer(2);
                        if let Ok(found) =
                            match_pre_hint(stack, newt, subject, hint_cell, formula, &cache)
                        {
                            res = found;
                            stack.pre_copy();
                            stack.preserve(&mut cache);
                            stack.preserve(&mut res);
                            stack.pop();
                        } else {
                            *(stack.stack_push()) = work_to_noun(Nock11ComputeResult);
                            push_formula(stack, hint_cell.tail());
                        }
                    } else {
                        panic!("IMPOSSIBLE: tried to compute a dynamic hint but hint is an atom");
                    }
                }
                Nock11ComputeResult => {
                    let hint = *stack.local_noun_pointer(1);
                    if let Ok(found) = match_post_hint(stack, newt, subject, hint, res) {
                        res = found;
                        stack.pre_copy();
                        stack.preserve(&mut cache);
                        stack.preserve(&mut res);
                        stack.pop();
                    } else {
                        *(stack.stack_push()) = work_to_noun(Nock11Done);
                        let formula = *stack.local_noun_pointer(2);
                        push_formula(stack, formula);
                    }
                }
                Nock11Done => {
                    let hint = *stack.local_noun_pointer(1);
                    let _ = match_post_hinted(stack, subject, hint, res, &mut cache);
                    stack.pre_copy();
                    stack.preserve(&mut cache);
                    stack.preserve(&mut res);
                    stack.pop();
                }
            };
        }
    });
    res
}

fn push_formula(stack: &mut NockStack, formula: Noun) {
    if let Ok(formula_cell) = formula.as_cell() {
        // Formula
        match formula_cell.head().as_either_atom_cell() {
            Right(_cell) => {
                unsafe {
                    *(stack.stack_push()) = formula_cell.tail();
                    *(stack.stack_push()) = formula_cell.head();
                    *(stack.stack_push()) = work_to_noun(NockCellComputeHead);
                }
            }
            Left(atom) => {
                if let Ok(direct) = atom.as_direct() {
                    match direct.data() {
                        0 => {
                            unsafe {
                                *(stack.stack_push()) = formula_cell.tail();
                                *(stack.stack_push()) = work_to_noun(Nock0Axis);
                            }
                        }
                        1 => {
                            unsafe {
                                *(stack.stack_push()) = formula_cell.tail();
                                *(stack.stack_push()) = work_to_noun(Nock1Constant);
                            };
                        }
                        2 => {
                            if let Ok(arg_cell) = formula_cell.tail().as_cell() {
                                stack.push(3);
                                unsafe {
                                    *(stack.stack_push()) =
                                        work_to_noun(Nock2ComputeSubject);

                                    //TODO use lw stack
                                    *(stack.local_noun_pointer(1)) = arg_cell.head();
                                    *(stack.local_noun_pointer(2)) = arg_cell.tail();
                                };
                            } else {
                                panic!("Argument for Nock 2 must be cell");
                            };
                        }
                        3 => {
                            unsafe {
                                *(stack.stack_push()) = formula_cell.tail();
                                *(stack.stack_push()) = work_to_noun(Nock3ComputeChild);
                            }
                        }
                        4 => {
                            unsafe {
                                *(stack.stack_push()) = formula_cell.tail();
                                *(stack.stack_push()) = work_to_noun(Nock4ComputeChild);
                            }
                        }
                        5 => {
                            if let Ok(arg_cell) = formula_cell.tail().as_cell() {
                                unsafe {
                                    *(stack.stack_push()) = arg_cell.tail();
                                    *(stack.stack_push()) = arg_cell.head();
                                    *(stack.stack_push()) = work_to_noun(Nock5ComputeLeftChild);
                                };
                            } else {
                                panic!("Argument for Nock 5 must be cell");
                            };
                        }
                        6 => {
                            if let Ok(arg_cell) = formula_cell.tail().as_cell() {
                                if let Ok(branch_cell) = arg_cell.tail().as_cell() {
                                    unsafe {
                                        *(stack.stack_push()) = branch_cell.tail();
                                        *(stack.stack_push()) = branch_cell.head();
                                        *(stack.stack_push()) = arg_cell.head();
                                        *(stack.stack_push()) = work_to_noun(Nock6ComputeTest);
                                    }
                                } else {
                                    panic!("Argument tail for Nock 6 must be cell");
                                };
                            } else {
                                panic!("Argument for Nock 6 must be cell");
                            }
                        }
                        7 => {
                            if let Ok(arg_cell) = formula_cell.tail().as_cell() {
                                unsafe {
                                    *(stack.stack_push()) = arg_cell.tail();
                                    *(stack.stack_push()) = arg_cell.head();
                                    *(stack.stack_push()) = work_to_noun(Nock7ComputeSubject);
                                }
                            } else {
                                panic!("Argument for Nock 7 must be cell");
                            };
                        }
                        8 => {
                            if let Ok(arg_cell) = formula_cell.tail().as_cell() {
                                unsafe {
                                    *(stack.stack_push()) = arg_cell.tail();
                                    *(stack.stack_push()) = arg_cell.head();
                                    *(stack.stack_push()) =
                                        work_to_noun(Nock8ComputeSubject);
                                };
                            } else {
                                panic!("Argument for Nock 8 must be cell");
                            };
                        }
                        9 => {
                            if let Ok(arg_cell) = formula_cell.tail().as_cell() {
                                stack.push(3);
                                unsafe {
                                    *(stack.stack_push()) = work_to_noun(Nock9ComputeCore);

                                    //TODO use lw stack
                                    *(stack.local_noun_pointer(1)) = arg_cell.head();
                                    *(stack.local_noun_pointer(2)) = arg_cell.tail();
                                };
                            } else {
                                panic!("Argument for Nock 9 must be cell");
                            };
                        }
                        10 => {
                            if let Ok(arg_cell) = formula_cell.tail().as_cell() {
                                if let Ok(patch_cell) = arg_cell.head().as_cell() {
                                    unsafe {
                                        *(stack.stack_push()) = patch_cell.head();
                                        *(stack.stack_push()) = patch_cell.tail();
                                        *(stack.stack_push()) = arg_cell.tail();
                                        *(stack.stack_push()) = work_to_noun(Nock10ComputeTree);
                                    };
                                } else {
                                    panic!("Argument head for Nock 10 must be cell");
                                };
                            } else {
                                panic!("Argument for Nock 10 must be cell");
                            };
                        }
                        11 => {
                            if let Ok(arg_cell) = formula_cell.tail().as_cell() {
                                stack.push(3);
                                unsafe {
                                    *(stack.stack_push()) =
                                        work_to_noun(if arg_cell.head().is_cell() {
                                            Nock11ComputeHint
                                        } else {
                                            Nock11ComputeResult
                                        });
                                    *(stack.local_noun_pointer(1)) = arg_cell.head();
                                    *(stack.local_noun_pointer(2)) = arg_cell.tail();
                                };
                            } else {
                                panic!("Argument for Nock 11 must be cell");
                            };
                        }
                        _ => {
                            panic!("Invalid opcode");
                        }
                    }
                } else {
                    panic!("Invalid opcode");
                }
            }
        }
    } else {
        panic!("Bad formula: atoms are not formulas: {}", formula);
    }
}

/** Note: axis must fit in a direct atom */
pub fn raw_slot(noun: Noun, axis: u64) -> Noun {
    slot(noun, DirectAtom::new(axis).unwrap().as_bitslice())
}

pub fn slot(mut noun: Noun, axis: &BitSlice<u64, Lsb0>) -> Noun {
    let mut cursor = if let Some(x) = axis.last_one() {
        x
    } else {
        panic!("0 is not allowed as an axis")
    };
    loop {
        if cursor == 0 {
            break;
        };
        cursor -= 1;
        if let Ok(cell) = noun.as_cell() {
            if axis[cursor] {
                noun = cell.tail();
            } else {
                noun = cell.head();
            }
        } else {
            panic!("Axis tried to descend through atom: {}", noun);
        };
    }
    noun
}

fn edit(
    stack: &mut NockStack,
    edit_axis: &BitSlice<u64, Lsb0>,
    patch: Noun,
    mut tree: Noun,
) -> Noun {
    let mut res = patch;
    let mut dest: *mut Noun = &mut res;
    let mut cursor = edit_axis
        .last_one()
        .expect("0 is not allowed as an edit axis");
    loop {
        if cursor == 0 {
            unsafe {
                *dest = patch;
            }
            break;
        };
        if let Ok(tree_cell) = tree.as_cell() {
            cursor -= 1;
            if edit_axis[cursor] {
                unsafe {
                    let (cell, cellmem) = Cell::new_raw_mut(stack);
                    *dest = cell.as_noun();
                    (*cellmem).head = tree_cell.head();
                    dest = &mut ((*cellmem).tail);
                }
                tree = tree_cell.tail();
            } else {
                unsafe {
                    let (cell, cellmem) = Cell::new_raw_mut(stack);
                    *dest = cell.as_noun();
                    (*cellmem).tail = tree_cell.tail();
                    dest = &mut ((*cellmem).head);
                }
                tree = tree_cell.head();
            }
        } else {
            panic!("Invalid axis for edit");
        };
    }
    res
}

pub fn inc(stack: &mut NockStack, atom: Atom) -> Atom {
    match atom.as_either() {
        Left(direct) => Atom::new(stack, direct.data() + 1),
        Right(indirect) => {
            let indirect_slice = indirect.as_bitslice();
            match indirect_slice.first_zero() {
                None => {
                    // all ones, make an indirect one word bigger
                    let (new_indirect, new_slice) =
                        unsafe { IndirectAtom::new_raw_mut_bitslice(stack, indirect.size() + 1) };
                    new_slice.set(indirect_slice.len(), true);
                    new_indirect.as_atom()
                }
                Some(first_zero) => {
                    let (new_indirect, new_slice) =
                        unsafe { IndirectAtom::new_raw_mut_bitslice(stack, indirect.size()) };
                    new_slice.set(first_zero, true);
                    new_slice[first_zero + 1..]
                        .copy_from_bitslice(&indirect_slice[first_zero + 1..]);
                    new_indirect.as_atom()
                }
            }
        }
    }
}

/** Match hints which apply before the formula is evaluated */
fn match_pre_hint(
    stack: &mut NockStack,
    newt: &mut Option<&mut Newt>,
    subject: Noun,
    cell: Cell,
    formula: Noun,
    cache: &Hamt<Noun>,
) -> Result<Noun, ()> {
    let direct = cell.head().as_direct()?;
    match direct.data() {
        // %sham hints are scaffolding until we have a real jet dashboard
        tas!(b"sham") => {
            let jet_formula = cell.tail().as_cell()?;
            let jet_name = jet_formula.tail();

            let jet = jets::get_jet(jet_name).ok_or(())?;
            if let Ok(mut jet_res) = jet(stack, subject) {
                // if in test mode, check that the jet returns the same result as the raw nock
                if jets::get_jet_test_mode(jet_name) {
                    let mut nock_res = interpret(stack, newt, subject, formula);
                    if unsafe { !unifying_equality(stack, &mut nock_res, &mut jet_res) } {
                        eprintln!(
                            "\rJet {} failed, raw: {}, jetted: {}",
                            jet_name, nock_res, jet_res
                        );
                        return Err(());
                    }
                }
                Ok(jet_res)
            } else {
                // Print jet errors and punt to Nock
                eprintln!("\rJet {} failed", jet_name);
                Err(())
            }
        }
        tas!(b"memo") => {
            let formula = unsafe { *stack.local_noun_pointer(2) };
            let mut key = Cell::new(stack, subject, formula).as_noun();
            if let Some(res) = cache.lookup(stack, &mut key) {
                Ok(res)
            } else {
                Err(())
            }
        }
        _ => Err(()),
    }
}

/** Match static hints and dynamic hints after they're evaluated */
fn match_post_hint(
    stack: &mut NockStack,
    newt: &mut Option<&mut Newt>,
    _subject: Noun,
    hint: Noun,
    res: Noun,
) -> Result<Noun, ()> {
    let direct = hint.as_cell()?.head().as_direct()?;
    match direct.data() {
        tas!(b"slog") => {
            let slog_cell = res.as_cell()?;
            let pri = slog_cell.head().as_direct()?.data();
            let tank = slog_cell.tail();
            if let Some(not) = newt {
                not.slog(stack, pri, tank);
            } else {
                println!("slog: {} {}", pri, tank);
            }
            Err(())
        }
        _ => Err(()),
    }
}

fn match_post_hinted(
    stack: &mut NockStack,
    subject: Noun,
    hint: Noun,
    res: Noun,
    cache: &mut Hamt<Noun>,
) -> Result<(), ()> {
    let direct = hint.as_cell()?.head().as_direct()?;
    match direct.data() {
        tas!(b"memo") => {
            let formula = unsafe { *stack.local_noun_pointer(2) };
            let mut key = Cell::new(stack, subject, formula).as_noun();
            *cache = cache.insert(stack, &mut key, res);
            Ok(())
        }
        _ => Err(()),
    }
}
