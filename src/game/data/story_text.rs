//! Story text and tables, generated from narr.asm by tools/narr2rs.py. Do not edit.
//!
//! Messages are NUL separated, with one more NUL at the end. A byte of 128 in a placard or
//! the title is a position escape (followed by x / 2 and y).

pub static EVENT_MSG: &[u8] =
    b"% was getting rather hungry.\x00% was getting very hungry.\x00% was starving!\x00% was gett\
        ing tired.\x00% was getting sleepy.\x00% was hit and killed!\x00% was drowned in the water!\
        \x00% was burned in the lava.\x00% was turned to stone by the witch.\x00% started the journ\
        ey in his home village of Tambry\x00as had his brother before him.\x00as had his brothers b\
        efore him.\x00% just couldn't stay awake any longer!\x00% was feeling quite full.\x00% was \
        feeling quite rested.\x00Even % would not be stupid enough to draw weapon in here.\x00A gre\
        at calming influence comes over %, preventing him from drawing his weapon.\x00% picked up a\
        \x20scrap of paper.\x00It read: \"Find the turtle!\"\x00It read: \"Meet me at midnight at t\
        he Crypt. Signed, the Wraith Lord.\"\x00% looked around but discovered nothing.\x00% does n\
        ot have that item.\x00% bought some food and ate it.\x00% bought some arrows.\x00% passed o\
        ut from hunger!\x00% is not sleepy.\x00% was tired, so he decided to lie down and sleep.\x00\
        % perished in the hot lava!\x00It was midnight.\x00It was morning.\x00It was midday.\x00Eve\
        ning was drawing near.\x00Ground is too hot for swan to land.\x00Flying too fast to dismoun\
        t.\x00\"They're all dead!\" he cried.\x00No time for that now!\x00% put an apple away for l\
        ater.\x00% ate one of his apples.\x00% discovered a hidden object.\x00\x00";

pub static PLACE_TBL: [u8; 87] = [
    51, 51, 19,
    64, 69, 2,
    70, 73, 3,
    80, 95, 6,
    96, 99, 7,
    138, 139, 8,
    144, 144, 9,
    147, 147, 10,
    148, 148, 20,
    159, 162, 17,
    163, 163, 18,
    164, 167, 12,
    168, 168, 21,
    170, 170, 22,
    171, 174, 14,
    176, 176, 13,
    178, 178, 23,
    179, 179, 24,
    180, 180, 25,
    175, 180, 0,
    208, 221, 11,
    243, 243, 16,
    250, 252, 0,
    255, 255, 26,
    78, 78, 4,
    187, 239, 4,
    0, 79, 0,
    185, 254, 15,
    0, 255, 0,
];

pub static INSIDE_TBL: [u8; 111] = [
    2, 2, 2,
    7, 7, 3,
    4, 4, 4,
    5, 6, 5,
    9, 10, 6,
    30, 30, 7,
    19, 33, 14,
    101, 101, 14,
    130, 134, 14,
    36, 36, 13,
    37, 42, 12,
    46, 46, 0,
    43, 59, 11,
    100, 100, 11,
    143, 149, 11,
    62, 62, 16,
    65, 66, 18,
    60, 78, 17,
    82, 82, 17,
    86, 87, 17,
    92, 92, 17,
    94, 95, 17,
    97, 99, 17,
    120, 120, 17,
    116, 119, 17,
    139, 141, 17,
    79, 96, 9,
    104, 104, 19,
    114, 114, 20,
    105, 115, 8,
    135, 138, 8,
    125, 125, 21,
    127, 127, 10,
    142, 142, 22,
    121, 129, 22,
    150, 161, 15,
    0, 255, 0,
];

pub static PLACE_MSG: &[u8] =
    b"\x00\x00% returned to the village of Tambry.\x00% came to Vermillion Manor.\x00% reached th\
        e Mountains of Frost\x00% reached the Plain of Grief.\x00% came to the city of Marheim.\x00\
        % came to the Witch's castle.\x00% came to the Graveyard.\x00% came to a great stone ring.\x00\
        % came to a watchtower.\x00% traveled to the great Bog.\x00% came to the Crystal Palace.\x00\
        % came to mysterious Pixle Grove.\x00% entered the Citadel of Doom.\x00% entered the Burnin\
        g Waste.\x00% found an oasis.\x00% came to the hidden city of Azal.\x00% discovered an outl\
        ying fort.\x00% came to a small keep.\x00% came to an old castle.\x00% came to a log cabin.\
        \x00% came to a dark stone tower.\x00% came to an isolated cabin.\x00% came to the Tombs of\
        \x20Hemsath.\x00% reached the Forbidden Keep.\x00% found a cave in the hillside.\x00\x00";

pub static INSIDE_MSG: &[u8] =
    b"\x00\x00% came to a small chamber.\x00% came to a large chamber.\x00% came to a long passag\
        eway.\x00% came to a twisting tunnel.\x00% came to a forked intersection.\x00He entered the\
        \x20keep.\x00He entered the castle.\x00He entered the castle of King Mar.\x00He entered the\
        \x20sanctuary of the temple.\x00% entered the Spirit Plane.\x00% came to a large room.\x00%\
        \x20came to an octagonal room.\x00% traveled along a stone corridor.\x00% came to a stone m\
        aze.\x00He entered a small building.\x00He entered the building.\x00He entered the tavern.\x00\
        He went inside the inn.\x00He entered the crypt.\x00He walked into the cabin.\x00He unlocke\
        d the door and entered.\x00\x00";

const MSG1: &[u8] =
    b"\x80\x0a\x1c   \"Rescue the Talisman!\"\x80\x0a'was the Mayor's plea.\x80\x0a2\"Only the Ta\
        lisman can\x80\x0a=protect our village from\x80\x0aHthe evil forces of the\x80\x0aSnight.\"\
        \x20And so Julian\x80\x0a^set out on his quest to\x80\x0airecover it.\x00\x00";

const MSG2: &[u8] =
    b"\x80\x0c,Unfortunately for Julian,\x80\x0c7his luck had run out.\x80\x0cBMany months passed\
        \x20and\x80\x0cMJulian did not return...\x00\x00";

const MSG3: &[u8] =
    b"\x80\x14,So Phillip set out,\x80\x147determined to find his\x80\x14Bbrother and complete\x80\
        \x14Mthe quest.\x00\x00";

const MSG4: &[u8] =
    b"\x80\x0c,But sadly, Phillip's\x80\x0c7cleverness could not save\x80\x0cBhim from the same f\
        ate\x80\x0cMas his older brother.\x00\x00";

const MSG5: &[u8] =
    b"\x80\x0f\x1eSo Kevin took up the\x80\x0f)quest, risking all, for\x80\x0f4the village had gr\
        own\x80\x0f?desperate. Young and\x80\x0fJinexperienced, his\x80\x0fUchances did not look\x80\
        \x0f`good.\x00\x00";

const MSG6: &[u8] =
    b"\x80\x0a\x1fAnd so ends our sad tale.\x80\x0c-The Lesson of the Story:\x80!XStay at Home!\x00\
        \x00";

const MSG7: &[u8] =
    b"\x80\x0e Having defeated the\x80\x0e+villanous Necromancer\x80\x0e6and recovered the\x80\x0e\
        ATalisman, \x00\x00";

const MSG7A: &[u8] =
    b"\x80\x0eLreturned to Marheim\x80\x0eWwhere he wed the\x80\x0ebprincess...\x00\x00";

const MSG8: &[u8] =
    b"\x80\x0a\x1a\x00\x00";

const MSG8A: &[u8] =
    b" had rescued Katra,\x80\x0a%Princess of Marheim.\x80\x0a0Though they had pledged\x80\x0a;th\
        eir love for each, \x80\x0aFother, \x00\x00";

const MSG8B: &[u8] =
    b" knew that\x80\x0aQ his quest could not\x80\x0a\\be forsaken.\x00\x00";

const MSG9: &[u8] =
    b"\x80\x0a!\x00\x00";

const MSG9A: &[u8] =
    b" had rescued Karla\x80\x0a,(Katra's sister), Princess\x80\x0a7of Marheim. Though they\x80\x0a\
        Bhad pledged their love\x80\x0aMfor each other, \x00\x00";

const MSG9B: &[u8] =
    b"\x80\x0aXknew that his quest\x80\x0accould not be forsaken.\x00\x00";

const MSG10: &[u8] =
    b"\x80\x0a\x1a\x00\x00";

const MSG10A: &[u8] =
    b" had rescued Kandy\x80\x0a%(Katra's and Karla's\x80\x0a0sister), Princess of\x80\x0a;Marhei\
        m. Though they had\x80\x0aFpledged their love for\x80\x0aQeach other, \x00\x00";

const MSG10B: &[u8] =
    b" knew \x80\x0a\\that his quest could\x80\x0agnot be forsaken.\x00\x00";

const MSG11: &[u8] =
    b"\x80#%After seeing the\x80\x110princess safely to her\x80\x11;home city, and with a\x80\x11\
        Fking's gift in gold,\x80\x11Q\x00\x00";

const MSG11A: &[u8] =
    b" once more set\x80\x11\\out on his quest.\x00\x00";

const MSG12: &[u8] =
    b"\x80@\x13So...\x80\x11AYou, game seeker, would guide the\x80\x05Kbrothers to their destiny?\
        \x20You would\x80\x05Uaid them and give directions? Answer,\x80\x05_then, these three quest\
        ions and prove\x80\x05iyour fitness to be their advisor:\x00\x00";

pub static SPEECHES: &[u8] =
    b"% attempted to communicate with the Ogre but a guttural snarl was the only response.\x00\"H\
        uman must die!\" said the goblin-man.\x00\"Doom!\" wailed the wraith.\x00A clattering of bo\
        nes was the only reply.\x00% knew that it is a waste of time to talk to a snake.\x00...\x00\
        There was no reply.\x00\"Die, foolish mortal!\" he said.\x00\"No need to shout, son!\" he s\
        aid.\x00\"Nice weather we're having, isn't it?\" queried the ranger.\x00\"Good luck, sonny!\
        \" said the ranger. \"Hope you win!\"\x00\"If you need to cross the lake\" said the ranger,\
        \x20\"There's a raft just north of here.\"\x00\"Would you like to buy something?\" said the\
        \x20tavern keeper. \"Or do you just need lodging for the night?\"\x00\"Good Morning.\" said\
        \x20the tavern keeper. \"Hope you slept well.\"\x00\"Have a drink!\" said the tavern keeper\
        .\"\x00\"State your business!\" said the guard.\x0d\"My business is with the king.\" stated\
        \x20%, respectfully.\x00\"Please, sir, rescue me from this horrible prison!\" pleaded the p\
        rincess.\x00\"I cannot help you, young man.\" said the king. \"My armies are decimated, and\
        \x20I fear that with the loss of my children, I have lost all hope.\"\x00\"Here is a writ d\
        esignating you as my official agent. Be sure and show this to the Priest before you leave M\
        arheim.\x00\"I'm afraid I cannot help you, young man. I already gave the golden statue to t\
        he other young man.\x00\"If you could rescue the king's daughter,\" said Lord Trane, \"The \
        King's courage would be restored.\"\x00\"Sorry, I have no use for it.\"\x00\"The dragon's c\
        ave is directly north of here.\" said the ranger.\"\x00\"Alms! Alms for the poor!\"\x00\"I \
        have a prophecy for you, m'lord.\" said the beggar. \"You must seek two women, one Good, on\
        e Evil.\"\x00\"Lovely Jewels, glint in the night - give to us the gift of Sight!\" he said.\
        \x00\"Where is the hidden city? How can you find it when you cannot even see it?\" said the\
        \x20beggar.\x00\"Kind deeds could gain thee a friend from the sea.\"\x00\"Seek the place th\
        at is darker than night - There you shall find your goal in sight!\" said the wizard, crypt\
        ically.\x00\"Like the eye itself, a crystal Orb can help to find things concealed.\"\x00\"T\
        he Witch lives in the dim forest of Grimwood, where the very trees are warped to her will. \
        Her gaze is Death!\"\x00\"Only the light of the Sun can destroy the Witch's Evil.\"\x00\"Th\
        e maiden you seek lies imprisoned in an unreachable castle surrounded by unclimbable mounta\
        ins.\"\x00\"Tame the golden beast and no mountain may deny you! But what rope could hold su\
        ch a creature?\"\x00\"Just what I needed!\" he said.\x00\"Away with you, young ruffian!\" s\
        aid the Wizard. \"Perhaps you can find some small animal to torment if that pleases you!\"\x00\
        \"You must seek your enemy on the spirit plane. It is hazardous in the extreme. Space may t\
        wist, and time itself may run backwards!\"\x00\"When you wish to travel quickly, seek the p\
        ower of the Stones.\" he said.\x00\"Since you are brave of heart, I shall Heal all your wou\
        nds.\"\x0dInstantly % felt much better.\x00\"Ah! You have a writ from the king. Here is one\
        \x20of the golden statues of Azal-Car-Ithil. Find all five and you'll find the vanishing ci\
        ty.\"\x00\"Repent, Sinner! Thou art an uncouth brute and I have no interest in your convers\
        ation!\"\x00\"Ho there, young traveler!\" said the black figure. \"None may enter the sacre\
        d shrine of the People who came Before!\"\x00\"Your prowess in battle is great.\" said the \
        Knight of Dreams. \"You have earned the right to enter and claim the prize.\"\x00\"So this \
        is the so-called Hero who has been sent to hinder my plans. Simply Pathetic. Well, try this\
        , young Fool!\"\x00% gasped. The Necromancer had been transformed into a normal man. All of\
        \x20his evil was gone.\x00\"%.\" said the Sorceress. \"Welcome. Here is one of the five gol\
        den figurines you will need.\"\x0d\"Thank you.\" said %.\x00\"Look into my eyes and Die!!\"\
        \x20hissed the witch.\x0d\"Not a chance!\" replied %\x00The Spectre spoke. \"HE has usurped\
        \x20my place as lord of undead. Bring me bones of the ancient King and I'll help you destro\
        y him.\"\x00% gave him the ancient bones.\x0d\"Good! That spirit now rests quietly in my ha\
        lls. Take this crystal shard.\"\x00\"%...\" said the apparition. \"I am the ghost of your d\
        ead brother. Find my bones -- there you will find some things you need.\x00% gave him some \
        gold coins. \x0d\"Why, thank you, young sir!\"\x00\"Sorry, but I have nothing to sell.\"\x00\
        \x00\"The dragon's cave is east of here.\" said the ranger.\"\x00\"The dragon's cave is wes\
        t of here.\" said the ranger.\"\x00\"The dragon's cave is south of here.\" said the ranger.\
        \"\x00\"Oh, thank you for saving my eggs, kind man!\" said the turtle. \"Take this seashell\
        \x20as a token of my gratitude.\"\x00\"Just hop on my back if you need a ride somewhere.\" \
        said the turtle.\x00\"Stupid fool, you can't hurt me with that!\"\x00\"Your magic won't wor\
        k here, fool!\"\x00The Sunstone has made the witch vulnerable!\x00\x00";

pub static TITLE_TEXT: &[u8] =
    b"\x80\x1c!\"The Faery Tale Adventure\"\x80\x14OAnimation, Programming & Music\x80LZby\x808eD\
        avid Joiner\x80\x14\xa0Copyright 1986 MicroIllusions \x00\x00";

pub static HINOR: [u8; 256] = [
    1, 255, 248, 255, 252, 3, 128, 0, 1, 255, 0, 7, 252, 3, 128, 0,
    7, 224, 240, 120, 63, 3, 128, 0, 25, 15, 224, 63, 196, 195, 128, 0,
    252, 127, 192, 31, 241, 255, 128, 0, 241, 159, 128, 15, 204, 127, 128, 0,
    231, 231, 0, 7, 63, 63, 128, 0, 207, 249, 128, 12, 255, 159, 128, 0,
    159, 254, 96, 51, 255, 207, 128, 0, 63, 7, 152, 207, 195, 231, 128, 0,
    112, 1, 231, 159, 0, 51, 128, 0, 0, 0, 120, 60, 0, 3, 128, 0,
    0, 0, 25, 48, 0, 0, 0, 0, 0, 0, 120, 12, 0, 3, 128, 0,
    120, 1, 199, 243, 0, 51, 128, 0, 63, 135, 56, 60, 195, 231, 128, 0,
    159, 252, 240, 31, 63, 207, 128, 0, 207, 243, 192, 15, 207, 159, 128, 0,
    231, 207, 128, 7, 243, 63, 128, 0, 241, 63, 128, 15, 252, 127, 128, 0,
    252, 127, 192, 31, 241, 48, 128, 0, 51, 31, 224, 63, 199, 192, 128, 0,
    15, 224, 240, 120, 63, 0, 128, 0, 3, 255, 0, 7, 255, 0, 128, 0,
    3, 255, 248, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub static HIVAR: [u8; 256] = [
    1, 255, 248, 255, 252, 3, 128, 0, 121, 255, 2, 7, 252, 243, 128, 0,
    103, 224, 242, 120, 63, 51, 128, 0, 25, 15, 231, 63, 196, 195, 128, 0,
    252, 127, 207, 159, 241, 255, 128, 0, 241, 159, 159, 207, 204, 127, 128, 0,
    231, 231, 63, 231, 63, 63, 128, 0, 207, 249, 159, 204, 255, 159, 128, 0,
    159, 254, 103, 51, 255, 207, 128, 0, 63, 7, 152, 207, 195, 231, 128, 0,
    112, 249, 231, 159, 60, 51, 128, 0, 15, 254, 120, 60, 255, 195, 128, 0,
    255, 255, 153, 51, 255, 248, 0, 0, 7, 254, 120, 12, 255, 195, 128, 0,
    120, 121, 199, 243, 60, 51, 128, 0, 63, 135, 56, 60, 195, 231, 128, 0,
    159, 252, 243, 159, 63, 207, 128, 0, 207, 243, 207, 207, 207, 159, 128, 0,
    231, 207, 159, 231, 243, 63, 128, 0, 241, 63, 159, 207, 252, 127, 128, 0,
    252, 127, 207, 159, 241, 48, 128, 0, 51, 31, 231, 63, 199, 204, 128, 0,
    207, 224, 242, 120, 63, 60, 128, 0, 243, 255, 2, 7, 255, 0, 128, 0,
    3, 255, 248, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

/// The story pages, in order.
pub static PLACARD_MSGS: [&[u8]; 20] = [
    MSG1,
    MSG2,
    MSG3,
    MSG4,
    MSG5,
    MSG6,
    MSG7,
    MSG7A,
    MSG8,
    MSG8A,
    MSG8B,
    MSG9,
    MSG9A,
    MSG9B,
    MSG10,
    MSG10A,
    MSG10B,
    MSG11,
    MSG11A,
    MSG12,
];
