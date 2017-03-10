use parsers::mucalculus::{MuFormula,find_children};
use std::cmp;


pub fn nesting_depth(mu: &MuFormula) -> u64 {
    match *mu {
        MuFormula::RecursionValue(_, _) | MuFormula::Bool(_, _) | MuFormula::Action(_, _) => {
            return 0;
        }

        MuFormula::Not(_, ref f) | MuFormula::DiamondOp (_,  _, ref f) | MuFormula::BoxOp (_,  _, ref f)  => {
            return nesting_depth(&f);
        }

        MuFormula::And(_, ref f, ref g) | MuFormula::Or(_, ref f, ref g) => {
            return cmp::max(nesting_depth(&f), nesting_depth(&g));
        }

        MuFormula::Mu(_, _, ref f) | MuFormula::Nu(_, _, ref f) => {
            return 1 + nesting_depth(&f);
        }

    }
}

pub fn alternation_depth(mu: &MuFormula) -> u64 {
    match *mu {
        MuFormula::RecursionValue(_, _) | MuFormula::Bool(_, _) | MuFormula::Action(_, _) => {
            return 0;
        }

        MuFormula::Not(_, ref f) | MuFormula::DiamondOp (_,  _, ref f) | MuFormula::BoxOp (_,  _, ref f)  => {
            return alternation_depth(&f);
        }

        MuFormula::And(_, ref f, ref g) | MuFormula::Or(_, ref f, ref g) => {
            return cmp::max(alternation_depth(&f), alternation_depth(&g));
        }

        MuFormula::Mu(_, _, _) => {
            let mut m = 0;
            for child in find_children(mu) {
                match child {
                    MuFormula::Nu(_, _, _) => {
                        let contender = alternation_depth(&child);
                        if contender > m {
                            m = contender;
                        }
                    }
                    _ => {}
                };
            }
            return 1 + m;
        }

        MuFormula::Nu(_, _, _) => {
            let mut m = 0;
            for child in find_children(mu) {
                match child {
                    MuFormula::Mu(_, _, _) => {
                        
                        let contender = alternation_depth(&child);
                        if contender > m {
                            m = contender;
                        }
                    }
                    _ => {}
                };
            }
            return 1 + m;
        }

    }
}

pub fn dependent_alternation_depth(mu: &MuFormula) -> u64 {
    match *mu {
        MuFormula::RecursionValue(_, _) | MuFormula::Bool(_, _) | MuFormula::Action(_, _) => {
            return 0;
        }

        MuFormula::Not(_, ref f) | MuFormula::DiamondOp (_,  _, ref f) | MuFormula::BoxOp (_,  _, ref f)  => {
            return dependent_alternation_depth(&f);
        }

        MuFormula::And(_, ref f, ref g) | MuFormula::Or(_, ref f, ref g) => {
            return cmp::max(dependent_alternation_depth(&f), dependent_alternation_depth(&g));
        }

        MuFormula::Mu(_, ref c1, _) => {
            let mut m = 0;
            for child in find_children(mu) {
                match child { 
                    MuFormula::Nu(_, _, _) => {
                        let mut does_occur = false;
                        for grandchild in find_children(&child) {
                            match grandchild {
                                MuFormula::RecursionValue(_, ref c2) => {
                                    if c1 == c2 {
                                        does_occur = true;
                                    }
                                }
                                _ => {}
                            }
                        }
                        if does_occur {
                            let contender = dependent_alternation_depth(&child);
                            if contender > m {
                                m = contender;
                            }
                        }
                    }
                    _ => {}
                };
            }
            return 1 + m;
        }

        MuFormula::Nu(_, ref c1, _) => {
            let mut m = 0;
            for child in find_children(mu) {
                match child { 
                    MuFormula::Mu(_, _, _) => {
                        let mut does_occur = false;
                        for grandchild in find_children(&child) {
                            match grandchild { 
                                MuFormula::RecursionValue(_, ref c2) => {
                                    if c1 == c2 {
                                        does_occur = true;
                                    }
                                }
                                _ => {}
                            }
                        }
                        if does_occur {
                            let contender = dependent_alternation_depth(&child);
                            if contender > m {
                                m = contender;
                            }
                        }
                    }
                    _ => {}
                };
            }
            return 1 + m;
        }

    }
}
