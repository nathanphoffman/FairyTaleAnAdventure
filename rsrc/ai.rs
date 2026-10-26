//! What each enemy wants to do each frame (`actor_ai`, from the main loop in csrc/fmain.c): picks a
//! goal and a tactic from how near the hero is and what the enemy carries, and starts or ends the
//! battle music.

use super::actors::{aftermath, do_tactic};
use super::asm_text::{prq, set_course};
use super::mood::setmood;
use super::random::{bitrand, rand2, rand4};
use super::types::Shape;
use super::actors::{turtle_eggs};
use super::globals::{actors_on_screen, anim_list, anix, battleflag, daynight, extn, goodfairy, hero_x, hero_y, leader, xtype};


// Actor kinds and states (csrc/ftale.h).
const CARRIER: i8 = 5;
const SETFIG: i8 = 4;
const FIGHTING: i8 = 0;
const WALKING: i8 = 12;
const STILL: i8 = 13;
const DEAD: i8 = 15;
const FALL: i8 = 22;
const SHOOT1: i8 = 24;
const SHOOT3: i8 = 25;

// Goals.
const ARCHER1: i16 = 3;
const ARCHER2: i16 = 4;
const FLEE: i16 = 5;
const STAND: i16 = 6;
const WAIT: i16 = 8;
const FOLLOWER: i16 = 9;
const CONFUSED: i16 = 10;

// Tactics.
const FRUST: i16 = 0;
const PURSUE: i16 = 1;
const FOLLOW: i32 = 2;
const RANDOM: i16 = 4;
const BACKUP: i32 = 5;
const EVADE: i16 = 6;
const SHOOT: i16 = 8;
const SHOOTFRUST: i16 = 9;
const EGG_SEEK: i16 = 10;

/// The first actors are the hero and the raft.
const FIRST_AI_ACTOR: usize = 2;
/// The good fairy is on screen below this countdown; nobody acts then.
const FAIRY_ARRIVES: u8 = 120;
/// An actor within this many pixels (in x and y) of the hero counts as on screen.
const SCREEN_RANGE: i16 = 300;
/// Special encounters (xtype above this) only fight if they are the encounter's race.
const SPECIAL_XTYPE: u16 = 59;
/// Carriers (the turtle) re-aim every 16 ticks, steering mode 5.
const CARRIER_AIM_MASK: u16 = 15;
const CARRIER_AIM_MODE: i32 = 5;
/// Weapon bit for a bow.
const BOW_BIT: i8 = 4;
/// Archers back off when closer than this, shoot inside the second pair, else close in.
const ARCHER_TOO_CLOSE: (i16, i16) = (40, 30);
const ARCHER_IN_RANGE: i16 = 70;
/// Distance at which a fighter closes to strike, minus its goal number (the dark knight: this).
const STRIKE_BASE: i16 = 14;
const DKNIGHT: u8 = 7;
const DKNIGHT_STRIKE: i16 = 16;
const SPIDER_EGGS_RACE: u8 = 4;
const WEAK: i16 = 6;
const NEARLY_DEAD: i16 = 2;
const DKNIGHT_FACING: i8 = 5;
const PRQ_STATS: i32 = 4;
const PRQ_VITALITY: i32 = 7;

/// Decides each enemy's goal and tactic, and starts or ends the battle mood.
pub fn actor_ai() {
    unsafe {
        actors_on_screen = 0;
        leader = 0;
        let battle2 = battleflag;
        battleflag = 0;
        let mut i = FIRST_AI_ACTOR;
        while i < anix as usize {
            if goodfairy != 0 && goodfairy < FAIRY_ARRIVES {
                break;
            }
            think(i, battle2);
            i += 1;
        }
        if battle2 == 0 && battleflag != 0 {
            setmood(1);
        }
        if battleflag == 0 && battle2 != 0 {
            prq(PRQ_VITALITY);
            prq(PRQ_STATS);
            aftermath();
        }
    }
}

/// One actor's decision. Returns early (skipping the goal update) for carriers, people and
/// anyone out of hit points who is on screen.
unsafe fn think(i: usize, battle2: i8) {
    unsafe {
        let an = &raw mut anim_list[i];
        if (*an).kind == CARRIER {
            if daynight & CARRIER_AIM_MASK == 0 {
                set_course(i as i32, hero_x as i32, hero_y as i32, CARRIER_AIM_MODE);
            }
            return;
        } else if (*an).kind == SETFIG {
            return;
        }
        let mut mode = (*an).goal as i16;
        let mut tactic = (*an).tactic as i16;

        let xd = ((*an).abs_x as i32 - hero_x as i32) as i16;
        let yd = ((*an).abs_y as i32 - hero_y as i32) as i16;
        let (xd, yd) = (xd.wrapping_abs(), yd.wrapping_abs());
        if xd < SCREEN_RANGE && yd < SCREEN_RANGE {
            actors_on_screen = 1;
            if (*an).vitality < 1 {
                return;
            }
            if (*an).visible != 0 || battle2 != 0 {
                battleflag = 1;
            }
        }
        let mut r = bitrand(15) == 0;
        let hero_state = anim_list[0].state;
        if hero_state == DEAD || hero_state == FALL {
            mode = if leader == 0 { FLEE } else { FOLLOWER };
        } else if (*an).vitality < NEARLY_DEAD
            || (xtype > SPECIAL_XTYPE && (*an).race != (*extn).v3)
        {
            mode = FLEE; // special encounters don't flee in the 60+ zones
        }

        if tactic == FRUST || tactic == SHOOTFRUST {
            if (*an).weapon & BOW_BIT != 0 {
                do_tactic(i as i32, rand4() + 2);
            } else {
                do_tactic(i as i32, rand2() + 3);
            }
        } else if (*an).state == SHOOT1 {
            (*an).state = SHOOT3;
        } else if mode <= ARCHER2 {
            if mode & 2 == 0 {
                r = rand4() == 0;
            }
            if r {
                (mode, tactic) = choose_tactic(an, mode, xd, yd);
            }
            hostile_action(i, an, mode, tactic, xd, yd);
        } else if mode == FLEE {
            do_tactic(i as i32, BACKUP);
        } else if mode == FOLLOWER {
            do_tactic(i as i32, FOLLOW);
        } else if mode == STAND {
            set_course(i as i32, hero_x as i32, hero_y as i32, 0);
            (*an).state = STILL;
        } else if mode == WAIT {
            (*an).state = STILL;
        }

        (*an).goal = mode as i8;
        if leader == 0 {
            leader = i as i16;
        }
    }
}

/// A hostile enemy picks a new tactic. May also change its mode to wandering if it has no weapon.
unsafe fn choose_tactic(an: *mut Shape, mut mode: i16, xd: i16, yd: i16) -> (i16, i16) {
    unsafe {
        let tactic;
        if (*an).race == SPIDER_EGGS_RACE && turtle_eggs != 0 {
            tactic = EGG_SEEK;
        } else if (*an).weapon < 1 {
            mode = CONFUSED;
            tactic = RANDOM;
        } else if (*an).vitality < WEAK && rand2() != 0 {
            tactic = EVADE;
        } else if mode >= ARCHER1 {
            tactic = if xd < ARCHER_TOO_CLOSE.0 && yd < ARCHER_TOO_CLOSE.1 {
                BACKUP as i16
            } else if xd < ARCHER_IN_RANGE && yd < ARCHER_IN_RANGE {
                SHOOT
            } else {
                PURSUE
            };
        } else {
            tactic = PURSUE;
        }
        (mode, tactic)
    }
}

/// A hostile enemy strikes if close enough, otherwise acts on its tactic.
unsafe fn hostile_action(i: usize, an: *mut Shape, mode: i16, tactic: i16, xd: i16, yd: i16) {
    unsafe {
        let mut thresh = STRIKE_BASE - mode;
        if (*an).race == DKNIGHT {
            thresh = DKNIGHT_STRIKE;
        }
        if (*an).weapon & BOW_BIT == 0 && xd < thresh && yd < thresh {
            set_course(i as i32, hero_x as i32, hero_y as i32, 0);
            if (*an).state >= WALKING {
                (*an).state = FIGHTING;
            }
        } else if (*an).race == DKNIGHT && (*an).vitality != 0 {
            (*an).state = STILL;
            (*an).facing = DKNIGHT_FACING;
        } else {
            do_tactic(i as i32, tactic as i32);
        }
    }
}
