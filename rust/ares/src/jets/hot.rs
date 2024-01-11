use crate::hamt::Hamt;
use crate::hamt::HamtIterator;
use crate::jets::*;
use crate::unifying_equality::unifying_equality;
use crate::mem::Preserve;
use crate::noun::{Atom, DirectAtom, IndirectAtom, Noun, D, T};
use ares_macros::tas;
use either::Either::{self, Left, Right};
use std::ptr::null_mut;

/** Root for Hoon %k.139
 */
pub const K_139: Either<&[u8], (u64, u64)> = Right((tas!(b"k"), 139));

// // This is the const state all in one spot as literals
// #[allow(clippy::complexity)]
// const SHAM_HOT_STATE: &[(&[Either<u64, (u64, u64)>], u64, Jet)] = &[
//     (&[A_50, Left(b"add")], 1, jet_add),
//     (&[A_50, Left(b"dec")], 1, jet_dec),
//     (&[A_50, Left(b"div")], 1, jet_div),
//     (&[A_50, Left(b"dvr")], 1, jet_dvr),
//     (&[A_50, Left(b"gth")], 1, jet_gth),
//     (&[A_50, Left(b"gte")], 1, jet_gte),
//     (&[A_50, Left(b"lte")], 1, jet_lte),
//     (&[A_50, Left(b"lth")], 1, jet_lth),
//     (&[A_50, Left(b"mod")], 1, jet_mod),
//     (&[A_50, Left(b"mul")], 1, jet_mul),
//     (&[A_50, Left(b"sub")], 1, jet_sub),
//     //
//     (&[A_50, Left(b"cap")], 1, jet_cap),
//     (&[A_50, Left(b"mas")], 1, jet_mas),
//     //
//     (&[A_50, Left(b"lent")], 1, jet_lent),
//     (&[A_50, Left(b"flop")], 1, jet_flop),
//     //
//     (&[A_50, Left(b"bex")], 1, jet_bex),
//     (&[A_50, Left(b"can")], 1, jet_can),
//     (&[A_50, Left(b"cat")], 1, jet_cat),
//     (&[A_50, Left(b"cut")], 1, jet_cut),
//     (&[A_50, Left(b"end")], 1, jet_end),
//     (&[A_50, Left(b"lsh")], 1, jet_lsh),
//     (&[A_50, Left(b"met")], 1, jet_met),
//     (&[A_50, Left(b"rap")], 1, jet_rap),
//     (&[A_50, Left(b"rep")], 1, jet_rep),
//     (&[A_50, Left(b"rev")], 1, jet_rev),
//     (&[A_50, Left(b"rip")], 1, jet_rip),
//     (&[A_50, Left(b"rsh")], 1, jet_rsh),
//     (&[A_50, Left(b"xeb")], 1, jet_xeb),
//     //
//     (&[A_50, Left(b"con")], 1, jet_con),
//     (&[A_50, Left(b"dis")], 1, jet_dis),
//     (&[A_50, Left(b"mix")], 1, jet_mix),
//     //
//     (&[A_50, Left(b"mug")], 1, jet_mug),
//     //
//     (&[A_50, Left(b"dor")], 1, jet_dor),
//     (&[A_50, Left(b"gor")], 1, jet_gor),
//     (&[A_50, Left(b"mor")], 1, jet_mor),
//     //
//     (&[A_50, Left(b"scow")], 1, jet_scow),
//     //
//     (&[A_50, Left(b"mink")], 1, jet_mink),
// ];

/**
 * (path, axis in battery, jet function pointer)
 * see the [Jet] typedef in ares::jets for the proper prototype
 */
pub type HotEntry = (&'static [Either<&'static [u8], (u64, u64)>], u64, Jet);

#[allow(clippy::complexity)]
pub const URBIT_HOT_STATE: &[HotEntry] = &[
    (&[K_139, Left(b"one"), Left(b"add")], 1, jet_add),
    (&[K_139, Left(b"one"), Left(b"dec")], 1, jet_dec),
    (&[K_139, Left(b"one"), Left(b"div")], 1, jet_div),
    (&[K_139, Left(b"one"), Left(b"dvr")], 1, jet_dvr),
    (&[K_139, Left(b"one"), Left(b"gte")], 1, jet_gte),
    (&[K_139, Left(b"one"), Left(b"gth")], 1, jet_gth),
    (&[K_139, Left(b"one"), Left(b"lte")], 1, jet_lte),
    (&[K_139, Left(b"one"), Left(b"lth")], 1, jet_lth),
    (&[K_139, Left(b"one"), Left(b"max")], 1, jet_max),
    (&[K_139, Left(b"one"), Left(b"min")], 1, jet_min),
    (&[K_139, Left(b"one"), Left(b"mod")], 1, jet_mod),
    (&[K_139, Left(b"one"), Left(b"mul")], 1, jet_mul),
    (&[K_139, Left(b"one"), Left(b"sub")], 1, jet_sub),
    //
    (&[K_139, Left(b"one"), Left(b"cap")], 1, jet_cap),
    (&[K_139, Left(b"one"), Left(b"mas")], 1, jet_mas),
    (&[K_139, Left(b"one"), Left(b"peg")], 1, jet_peg),
    //
    (
        &[K_139, Left(b"one"), Left(b"two"), Left(b"flop")],
        1,
        jet_flop,
    ),
    (
        &[K_139, Left(b"one"), Left(b"two"), Left(b"lent")],
        1,
        jet_lent,
    ),
    (
        &[K_139, Left(b"one"), Left(b"two"), Left(b"zing")],
        1,
        jet_zing,
    ),
    //
    (
        &[K_139, Left(b"one"), Left(b"two"), Left(b"bex")],
        1,
        jet_bex,
    ),
    (
        &[K_139, Left(b"one"), Left(b"two"), Left(b"can")],
        1,
        jet_can,
    ),
    (
        &[K_139, Left(b"one"), Left(b"two"), Left(b"cat")],
        1,
        jet_cat,
    ),
    (
        &[K_139, Left(b"one"), Left(b"two"), Left(b"cut")],
        1,
        jet_cut,
    ),
    (
        &[K_139, Left(b"one"), Left(b"two"), Left(b"end")],
        1,
        jet_end,
    ),
    (
        &[K_139, Left(b"one"), Left(b"two"), Left(b"lsh")],
        1,
        jet_lsh,
    ),
    (
        &[K_139, Left(b"one"), Left(b"two"), Left(b"met")],
        1,
        jet_met,
    ),
    (
        &[K_139, Left(b"one"), Left(b"two"), Left(b"rap")],
        1,
        jet_rap,
    ),
    (
        &[K_139, Left(b"one"), Left(b"two"), Left(b"rep")],
        1,
        jet_rep,
    ),
    (
        &[K_139, Left(b"one"), Left(b"two"), Left(b"rev")],
        1,
        jet_rev,
    ),
    (
        &[K_139, Left(b"one"), Left(b"two"), Left(b"rip")],
        1,
        jet_rip,
    ),
    (
        &[K_139, Left(b"one"), Left(b"two"), Left(b"rsh")],
        1,
        jet_rsh,
    ),
    (
        &[K_139, Left(b"one"), Left(b"two"), Left(b"xeb")],
        1,
        jet_xeb,
    ),
    //
    (
        &[K_139, Left(b"one"), Left(b"two"), Left(b"con")],
        1,
        jet_con,
    ),
    (
        &[K_139, Left(b"one"), Left(b"two"), Left(b"dis")],
        1,
        jet_dis,
    ),
    (
        &[K_139, Left(b"one"), Left(b"two"), Left(b"mix")],
        1,
        jet_mix,
    ),
    //
    (
        &[K_139, Left(b"one"), Left(b"two"), Left(b"mug")],
        1,
        jet_mug,
    ),
    //
    (
        &[K_139, Left(b"one"), Left(b"two"), Left(b"dor")],
        1,
        jet_dor,
    ),
    (
        &[K_139, Left(b"one"), Left(b"two"), Left(b"gor")],
        1,
        jet_gor,
    ),
    (
        &[K_139, Left(b"one"), Left(b"two"), Left(b"mor")],
        1,
        jet_mor,
    ),
    //
    (
        &[K_139, Left(b"one"), Left(b"two"), Left(b"cue")],
        1,
        jet_cue,
    ),
    (
        &[K_139, Left(b"one"), Left(b"two"), Left(b"jam")],
        1,
        jet_jam,
    ),
    //
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"shas"),
        ],
        1,
        jet_shas,
    ),
    //
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"shax"),
        ],
        1,
        jet_shax,
    ),
    //
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"shay"),
        ],
        1,
        jet_shay,
    ),
    //
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"shal"),
        ],
        1,
        jet_shal,
    ),
    //
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"sha"),
            Left(b"sha1"),
        ],
        1,
        jet_sha1,
    ),
    //
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"last"),
        ],
        1,
        jet_last,
    ),
    //
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"bend"),
            Left(b"fun"),
        ],
        1,
        jet_bend,
    ),
    //
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"comp"),
            Left(b"fun"),
        ],
        1,
        jet_comp,
    ),
    //
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"glue"),
            Left(b"fun"),
        ],
        1,
        jet_glue,
    ),
    //
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pfix"),
        ],
        1,
        jet_pfix,
    ),
    //
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pose"),
        ],
        1,
        jet_pose,
    ),
    //
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"sfix"),
        ],
        1,
        jet_sfix,
    ),
    //
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"easy"),
            Left(b"fun"),
        ],
        1,
        jet_easy,
    ),
    //
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"here"),
            Left(b"fun"),
        ],
        1,
        jet_here,
    ),
    //
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"just"),
            Left(b"fun"),
        ],
        1,
        jet_just,
    ),
    //
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"mask"),
            Left(b"fun"),
        ],
        1,
        jet_mask,
    ),
    //
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"stag"),
            Left(b"fun"),
        ],
        1,
        jet_stag,
    ),
    //
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"scow"),
        ],
        1,
        jet_scow,
    ),
    //
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"mink"),
        ],
        1,
        jet_mink,
    ),
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"mole"),
        ],
        1,
        jet_mole,
    ),
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"mule"),
        ],
        1,
        jet_mule,
    ),
    //
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"ut"),
            Left(b"crop"),
        ],
        1,
        jet_ut_crop,
    ),
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"ut"),
            Left(b"fish"),
        ],
        1,
        jet_ut_fish,
    ),
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"ut"),
            Left(b"fuse"),
        ],
        1,
        jet_ut_fuse,
    ),
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"ut"),
            Left(b"mint"),
        ],
        1,
        jet_ut_mint,
    ),
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"ut"),
            Left(b"mull"),
        ],
        1,
        jet_ut_mull,
    ),
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"ut"),
            Left(b"nest"),
            Left(b"nest-in"),
            Left(b"nest-dext"),
        ],
        1,
        jet_ut_nest_dext,
    ),
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"ut"),
            Left(b"rest"),
        ],
        1,
        jet_ut_rest,
    ),
    //
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"hex"),
            Left(b"coed"),
            Left(b"ed"),
            Left(b"puck"),
        ],
        1,
        jet_puck,
    ),
    //
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"hex"),
            Left(b"coed"),
            Left(b"ed"),
            Left(b"shar"),
        ],
        1,
        jet_shar,
    ),
    //
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"hex"),
            Left(b"coed"),
            Left(b"ed"),
            Left(b"sign"),
        ],
        1,
        jet_sign,
    ),
    //
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"hex"),
            Left(b"coed"),
            Left(b"ed"),
            Left(b"veri"),
        ],
        1,
        jet_veri,
    ),
    //
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"hex"),
            Left(b"aes"),
            Left(b"siva"),
            Left(b"en"),
        ],
        1,
        jet_siva_en,
    ),
    //
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"hex"),
            Left(b"aes"),
            Left(b"siva"),
            Left(b"de"),
        ],
        1,
        jet_siva_de,
    ),
    //
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"hex"),
            Left(b"aes"),
            Left(b"sivb"),
            Left(b"en"),
        ],
        1,
        jet_sivb_en,
    ),
    //
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"hex"),
            Left(b"aes"),
            Left(b"sivb"),
            Left(b"de"),
        ],
        1,
        jet_sivb_de,
    ),
    //
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"hex"),
            Left(b"aes"),
            Left(b"sivc"),
            Left(b"en"),
        ],
        1,
        jet_sivc_en,
    ),
    //
    (
        &[
            K_139,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"hex"),
            Left(b"aes"),
            Left(b"sivc"),
            Left(b"de"),
        ],
        1,
        jet_sivc_de,
    ),
];

#[derive(Copy, Clone)]
pub struct Hot(Hamt<HotMemEntry>);

impl IntoIterator for Hot {
    type Item = (Noun, HotMemEntry);
    type IntoIter = HamtIterator<HotMemEntry>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Hot {
    pub fn init(stack: &mut NockStack, constant_hot_state: &[HotEntry]) -> Self {
        stack.frame_push(0); // we can preserve the hamt when we're done thus cleaning intermediate
                             // allocations
        unsafe {
            let mut hamt = Hamt::new(stack);
            for (htap, axe, jet) in constant_hot_state {
                let mut a_path = D(0);
                for i in *htap {
                    match i {
                        Left(tas) => {
                            let chum = IndirectAtom::new_raw_bytes_ref(stack, tas)
                                .normalize_as_atom()
                                .as_noun();
                            a_path = T(stack, &[chum, a_path]);
                        }
                        Right((tas, ver)) => {
                            let chum = T(
                                stack,
                                &[
                                    DirectAtom::new_panic(*tas).as_atom().as_noun(),
                                    DirectAtom::new_panic(*ver).as_atom().as_noun(),
                                ],
                            );
                            a_path = T(stack, &[chum, a_path]);
                        }
                    };
                }
                let axis = DirectAtom::new_panic(*axe).as_atom();
                let current_hot_entry = hamt
                    .lookup(stack, &mut a_path)
                    .unwrap_or(HotMemEntry(null_mut()));
                let hot_mem_ptr: *mut HotMem = stack.struct_alloc(1);
                *hot_mem_ptr = HotMem {
                    axis,
                    jet: *jet,
                    next: current_hot_entry,
                };
                hamt = hamt.insert(stack, &mut a_path, HotMemEntry(hot_mem_ptr));
            }
            stack.preserve(&mut hamt);
            stack.frame_pop();
            Hot(hamt)
        }
    }

    pub fn lookup(&mut self, stack: &mut NockStack, path: &mut Noun, axis: Atom) -> Option<Jet> {
        let he = self.0.lookup(stack, path)?;
        for (hot_axis, jet) in he {
            if unsafe { unifying_equality(stack, &mut hot_axis.as_noun(), &mut axis.as_noun()) } {
                return Some(jet);
            }
        }
        None
    }
}

impl Iterator for HotMemEntry {
    type Item = (Atom, Jet); // path,axis,jet
    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_null() {
            return None;
        }
        unsafe {
            let res = ((*(self.0)).axis, (*(self.0)).jet);
            *self = (*(self.0)).next;
            Some(res)
        }
    }
}

#[derive(Copy, Clone)]
pub struct HotMemEntry(*mut HotMem);

#[derive(Copy, Clone)]
struct HotMem {
    axis: Atom, // Axis of jetted formula in *battery*;
    jet: Jet,
    next: HotMemEntry,
}

impl Preserve for HotMemEntry {
    unsafe fn preserve(&mut self, stack: &mut NockStack) {
        let mut dest = self as *mut Self;

        loop {
            stack.preserve(&mut (*((*dest).0)).axis);
            let ptr = stack.struct_alloc_in_previous_frame::<HotMem>(1);
            *ptr = *((*dest).0);

            if (*ptr).next.0.is_null() {
                break;
            }
            *dest = HotMemEntry(ptr);
            dest = &mut ((*ptr).next)
        }
    }

    unsafe fn assert_in_stack(&self, stack: &NockStack) {
        let mut i = self;

        loop {
            if i.0.is_null() {
                break;
            }
            stack.assert_struct_is_in(i.0, 1);
            (*i.0).axis.as_noun().assert_in_stack(stack);
            i = &(*i.0).next;
        }
    }
}

impl Preserve for Hot {
    unsafe fn preserve(&mut self, stack: &mut NockStack) {
        self.0.preserve(stack);
    }

    unsafe fn assert_in_stack(&self, stack: &NockStack) {
        self.0.assert_in_stack(stack);
    }
}
