// Generated from the Rust tables by a one-off script, then kept by hand.
using System.Text;
using FaeryTale.Game.Data;

namespace FaeryTale.Game.Data;

public static partial class StoryText
{
    private static byte[] Latin1(string text) => Encoding.Latin1.GetBytes(text);

    public static readonly byte[] EVENT_MSG = Latin1("% was getting rather hungry.\u0000% was getting very hungry.\u0000% was starving!\u0000% was getting tired.\u0000% was getting sleepy.\u0000% was hit and killed!\u0000% was drowned in the water!\u0000% was burned in the lava.\u0000% was turned to stone by the witch.\u0000% started the journey in his home village of Tambry\u0000as had his brother before him.\u0000as had his brothers before him.\u0000% just couldn't stay awake any longer!\u0000% was feeling quite full.\u0000% was feeling quite rested.\u0000Even % would not be stupid enough to draw weapon in here.\u0000A great calming influence comes over %, preventing him from drawing his weapon.\u0000% picked up a scrap of paper.\u0000It read: \"Find the turtle!\"\u0000It read: \"Meet me at midnight at the Crypt. Signed, the Wraith Lord.\"\u0000% looked around but discovered nothing.\u0000% does not have that item.\u0000% bought some food and ate it.\u0000% bought some arrows.\u0000% passed out from hunger!\u0000% is not sleepy.\u0000% was tired, so he decided to lie down and sleep.\u0000% perished in the hot lava!\u0000It was midnight.\u0000It was morning.\u0000It was midday.\u0000Evening was drawing near.\u0000Ground is too hot for swan to land.\u0000Flying too fast to dismount.\u0000\"They're all dead!\" he cried.\u0000No time for that now!\u0000% put an apple away for later.\u0000% ate one of his apples.\u0000% discovered a hidden object.\u0000\u0000");

    public static readonly byte[] PLACE_TBL = [
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

    public static readonly byte[] INSIDE_TBL = [
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

    public static readonly byte[] PLACE_MSG = Latin1("\u0000\u0000% returned to the village of Tambry.\u0000% came to Vermillion Manor.\u0000% reached the Mountains of Frost\u0000% reached the Plain of Grief.\u0000% came to the city of Marheim.\u0000% came to the Witch's castle.\u0000% came to the Graveyard.\u0000% came to a great stone ring.\u0000% came to a watchtower.\u0000% traveled to the great Bog.\u0000% came to the Crystal Palace.\u0000% came to mysterious Pixle Grove.\u0000% entered the Citadel of Doom.\u0000% entered the Burning Waste.\u0000% found an oasis.\u0000% came to the hidden city of Azal.\u0000% discovered an outlying fort.\u0000% came to a small keep.\u0000% came to an old castle.\u0000% came to a log cabin.\u0000% came to a dark stone tower.\u0000% came to an isolated cabin.\u0000% came to the Tombs of Hemsath.\u0000% reached the Forbidden Keep.\u0000% found a cave in the hillside.\u0000\u0000");

    public static readonly byte[] INSIDE_MSG = Latin1("\u0000\u0000% came to a small chamber.\u0000% came to a large chamber.\u0000% came to a long passageway.\u0000% came to a twisting tunnel.\u0000% came to a forked intersection.\u0000He entered the keep.\u0000He entered the castle.\u0000He entered the castle of King Mar.\u0000He entered the sanctuary of the temple.\u0000% entered the Spirit Plane.\u0000% came to a large room.\u0000% came to an octagonal room.\u0000% traveled along a stone corridor.\u0000% came to a stone maze.\u0000He entered a small building.\u0000He entered the building.\u0000He entered the tavern.\u0000He went inside the inn.\u0000He entered the crypt.\u0000He walked into the cabin.\u0000He unlocked the door and entered.\u0000\u0000");

    private static readonly byte[] MSG1 = Latin1("\u0080\u000a\u001c   \"Rescue the Talisman!\"\u0080\u000a'was the Mayor's plea.\u0080\u000a2\"Only the Talisman can\u0080\u000a=protect our village from\u0080\u000aHthe evil forces of the\u0080\u000aSnight.\" And so Julian\u0080\u000a^set output on his quest to\u0080\u000airecover it.\u0000\u0000");

    private static readonly byte[] MSG2 = Latin1("\u0080\u000c,Unfortunately for Julian,\u0080\u000c7his luck had run out.\u0080\u000cBMany months passed and\u0080\u000cMJulian did not return...\u0000\u0000");

    private static readonly byte[] MSG3 = Latin1("\u0080\u0014,So Phillip set out,\u0080\u00147determined to find his\u0080\u0014Bbrother and complete\u0080\u0014Mthe quest.\u0000\u0000");

    private static readonly byte[] MSG4 = Latin1("\u0080\u000c,But sadly, Phillip's\u0080\u000c7cleverness could not save\u0080\u000cBhim from the same fate\u0080\u000cMas his older brother.\u0000\u0000");

    private static readonly byte[] MSG5 = Latin1("\u0080\u000f\u001eSo Kevin took up the\u0080\u000f)quest, risking all, for\u0080\u000f4the village had grown\u0080\u000f?desperate. Young and\u0080\u000fJinexperienced, his\u0080\u000fUchances did not look\u0080\u000f`good.\u0000\u0000");

    private static readonly byte[] MSG6 = Latin1("\u0080\u000a\u001fAnd so ends our sad tale.\u0080\u000c-The Lesson of the Story:\u0080!XStay at Home!\u0000\u0000");

    private static readonly byte[] MSG7 = Latin1("\u0080\u000e Having defeated the\u0080\u000e+villanous Necromancer\u0080\u000e6and recovered the\u0080\u000eATalisman, \u0000\u0000");

    private static readonly byte[] MSG7A = Latin1("\u0080\u000eLreturned to Marheim\u0080\u000eWwhere he wed the\u0080\u000ebprincess...\u0000\u0000");

    private static readonly byte[] MSG8 = Latin1("\u0080\u000a\u001a\u0000\u0000");

    private static readonly byte[] MSG8A = Latin1(" had rescued Katra,\u0080\u000a%Princess of Marheim.\u0080\u000a0Though they had pledged\u0080\u000a;their love for each, \u0080\u000aFother, \u0000\u0000");

    private static readonly byte[] MSG8B = Latin1(" knew that\u0080\u000aQ his quest could not\u0080\u000a\\be forsaken.\u0000\u0000");

    private static readonly byte[] MSG9 = Latin1("\u0080\u000a!\u0000\u0000");

    private static readonly byte[] MSG9A = Latin1(" had rescued Karla\u0080\u000a,(Katra's sister), Princess\u0080\u000a7of Marheim. Though they\u0080\u000aBhad pledged their love\u0080\u000aMfor each other, \u0000\u0000");

    private static readonly byte[] MSG9B = Latin1("\u0080\u000aXknew that his quest\u0080\u000accould not be forsaken.\u0000\u0000");

    private static readonly byte[] MSG10 = Latin1("\u0080\u000a\u001a\u0000\u0000");

    private static readonly byte[] MSG10A = Latin1(" had rescued Kandy\u0080\u000a%(Katra's and Karla's\u0080\u000a0sister), Princess of\u0080\u000a;Marheim. Though they had\u0080\u000aFpledged their love for\u0080\u000aQeach other, \u0000\u0000");

    private static readonly byte[] MSG10B = Latin1(" knew \u0080\u000a\\that his quest could\u0080\u000agnot be forsaken.\u0000\u0000");

    private static readonly byte[] MSG11 = Latin1("\u0080#%After seeing the\u0080\u00110princess safely to her\u0080\u0011;home city, and with a\u0080\u0011Fking's gift in gold,\u0080\u0011Q\u0000\u0000");

    private static readonly byte[] MSG11A = Latin1(" once more set\u0080\u0011\\out on his quest.\u0000\u0000");

    private static readonly byte[] MSG12 = Latin1("\u0080@\u0013So...\u0080\u0011AYou, game seeker, would guide the\u0080\u0005Kbrothers to their destiny? You would\u0080\u0005Uaid them and give directions? Answer,\u0080\u0005_then, these three questions and prove\u0080\u0005iyour fitness to be their advisor:\u0000\u0000");

    public static readonly byte[] SPEECHES = Latin1("% attempted to communicate with the Ogre but a guttural snarl was the only response.\u0000\"Human must die!\" said the goblin-man.\u0000\"Doom!\" wailed the wraith.\u0000A clattering of bones was the only reply.\u0000% knew that it is a waste of time to talk to a snake.\u0000...\u0000There was no reply.\u0000\"Die, foolish mortal!\" he said.\u0000\"No need to shout, son!\" he said.\u0000\"Nice weather we're having, isn't it?\" queried the ranger.\u0000\"Good luck, sonny!\" said the ranger. \"Hope you win!\"\u0000\"If you need to cross the lake\" said the ranger, \"There's a raft just north of here.\"\u0000\"Would you like to buy something?\" said the tavern keeper. \"Or do you just need lodging for the night?\"\u0000\"Good Morning.\" said the tavern keeper. \"Hope you slept well.\"\u0000\"Have a drink!\" said the tavern keeper.\"\u0000\"State your business!\" said the guard.\u000d\"My business is with the king.\" stated %, respectfully.\u0000\"Please, sir, rescue me from this horrible prison!\" pleaded the princess.\u0000\"I cannot help you, young man.\" said the king. \"My armies are decimated, and I fear that with the loss of my children, I have lost all hope.\"\u0000\"Here is a writ designating you as my official agent. Be sure and show this to the Priest before you leave Marheim.\u0000\"I'm afraid I cannot help you, young man. I already gave the golden statue to the other young man.\u0000\"If you could rescue the king's daughter,\" said Lord Trane, \"The King's courage would be restored.\"\u0000\"Sorry, I have no use for it.\"\u0000\"The dragon's cave is directly north of here.\" said the ranger.\"\u0000\"Alms! Alms for the poor!\"\u0000\"I have a prophecy for you, m'lord.\" said the beggar. \"You must seek two women, one Good, one Evil.\"\u0000\"Lovely Jewels, glint in the night - give to us the gift of Sight!\" he said.\u0000\"Where is the hidden city? How can you find it when you cannot even see it?\" said the beggar.\u0000\"Kind deeds could gain thee a friend from the sea.\"\u0000\"Seek the place that is darker than night - There you shall find your goal in sight!\" said the wizard, cryptically.\u0000\"Like the eye itself, a crystal Orb can help to find things concealed.\"\u0000\"The Witch lives in the dim forest of Grimwood, where the very trees are warped to her will. Her gaze is Death!\"\u0000\"Only the light of the Sun can destroy the Witch's Evil.\"\u0000\"The maiden you seek lies imprisoned in an unreachable castle surrounded by unclimbable mountains.\"\u0000\"Tame the golden beast and no mountain may deny you! But what rope could hold such a creature?\"\u0000\"Just what I needed!\" he said.\u0000\"Away with you, young ruffian!\" said the Wizard. \"Perhaps you can find some small animal to torment if that pleases you!\"\u0000\"You must seek your enemy on the spirit plane. It is hazardous in the extreme. Space may twist, and time itself may run backwards!\"\u0000\"When you wish to travel quickly, seek the power of the Stones.\" he said.\u0000\"Since you are brave of heart, I shall Heal all your wounds.\"\u000dInstantly % felt much better.\u0000\"Ah! You have a writ from the king. Here is one of the golden statues of Azal-Car-Ithil. Find all five and you'll find the vanishing city.\"\u0000\"Repent, Sinner! Thou art an uncouth brute and I have no interest in your conversation!\"\u0000\"Ho there, young traveler!\" said the black figure. \"None may enter the sacred shrine of the People who came Before!\"\u0000\"Your prowess in battle is great.\" said the Knight of Dreams. \"You have earned the right to enter and claim the prize.\"\u0000\"So this is the so-called Hero who has been sent to hinder my plans. Simply Pathetic. Well, try this, young Fool!\"\u0000% gasped. The Necromancer had been transformed into a normal man. All of his evil was gone.\u0000\"%.\" said the Sorceress. \"Welcome. Here is one of the five golden figurines you will need.\"\u000d\"Thank you.\" said %.\u0000\"Look into my eyes and Die!!\" hissed the witch.\u000d\"Not a chance!\" replied %\u0000The Spectre spoke. \"HE has usurped my place as lord of undead. Bring me bones of the ancient King and I'll help you destroy him.\"\u0000% gave him the ancient bones.\u000d\"Good! That spirit now rests quietly in my halls. Take this crystal shard.\"\u0000\"%...\" said the apparition. \"I am the ghost of your dead brother. Find my bones -- there you will find some things you need.\u0000% gave him some gold coins. \u000d\"Why, thank you, young sir!\"\u0000\"Sorry, but I have nothing to sell.\"\u0000\u0000\"The dragon's cave is east of here.\" said the ranger.\"\u0000\"The dragon's cave is west of here.\" said the ranger.\"\u0000\"The dragon's cave is south of here.\" said the ranger.\"\u0000\"Oh, thank you for saving my eggs, kind man!\" said the turtle. \"Take this seashell as a token of my gratitude.\"\u0000\"Just hop on my back if you need a ride somewhere.\" said the turtle.\u0000\"Stupid fool, you can't hurt me with that!\"\u0000\"Your magic won't work here, fool!\"\u0000The Sunstone has made the witch vulnerable!\u0000\u0000");

    public static readonly byte[] TITLE_TEXT = Latin1("\u0080\u001c!\"The Faery Tale Adventure\"\u0080\u0014OAnimation, Programming & Music\u0080LZby\u00808eDavid Joiner\u0080\u0014\u00a0Copyright 1986 MicroIllusions \u0000\u0000");

    public static readonly byte[] HINOR = [
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

    public static readonly byte[] HIVAR = [
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

    // The story pages, in order.
    public static readonly byte[][] PLACARD_MSGS = [
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
}
