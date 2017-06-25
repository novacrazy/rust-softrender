macro_rules! tuple_impls {
    ($(
        $Tuple:ident {
            $(($idx:tt) -> $T:ident)+
        }
    )+) => {
        $(
            impl<$($T),+> $crate::attachments::Color for ($($T,)+) where $($T: $crate::attachments::Color,)+ {
                type Alpha = ($(<$T as $crate::attachments::Color>::Alpha,)+);

                fn empty() -> Self {
                    ($(<$T as $crate::attachments::Color>::empty(),)+)
                }

                fn with_alpha(self, alpha: Self::Alpha) -> Self {
                    ($(<$T as $crate::attachments::Color>::with_alpha(self.$idx, alpha.$idx),)+)
                }

                fn mul_alpha(self, alpha: Self::Alpha) -> Self {
                    ($(<$T as $crate::attachments::Color>::mul_alpha(self.$idx, alpha.$idx),)+)
                }

                fn get_alpha(&self) -> Self::Alpha {
                    ($(<$T as $crate::attachments::Color>::get_alpha(&self.$idx),)+)
                }
            }

            impl<$($T),+> $crate::Interpolate for ($($T,)+) where $($T: $crate::Interpolate,)+ {
                fn barycentric_interpolate(u: f32, ux: &Self, v: f32, vx: &Self, w: f32, wx: &Self) -> Self{
                    ($($crate::Interpolate::barycentric_interpolate(u, &ux.$idx,
                                                                    v, &vx.$idx,
                                                                    w, &wx.$idx),)+)
                }

                fn linear_interpolate(t: f32, x1: &Self, x2: &Self) -> Self {
                    ($($crate::Interpolate::linear_interpolate(t, &x1.$idx, &x2.$idx),)+)
                }
            }
        )+
    }
}

tuple_impls! {
    Tuple1 {
        (0) -> A
    }
    Tuple2 {
        (0) -> A
        (1) -> B
    }
    Tuple3 {
        (0) -> A
        (1) -> B
        (2) -> C
    }
    Tuple4 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
    }
    Tuple5 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
    }
    Tuple6 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
    }
    Tuple7 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
    }
    Tuple8 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
        (7) -> H
    }
    Tuple9 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
        (7) -> H
        (8) -> I
    }
    Tuple10 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
        (7) -> H
        (8) -> I
        (9) -> J
    }
    Tuple11 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
        (7) -> H
        (8) -> I
        (9) -> J
        (10) -> K
    }
    Tuple12 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
        (7) -> H
        (8) -> I
        (9) -> J
        (10) -> K
        (11) -> L
    }
}