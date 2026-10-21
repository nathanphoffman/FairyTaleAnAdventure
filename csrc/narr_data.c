/* narr_data.c - story text and tables, generated from narr.asm by tools/narr2c.py. Do not edit. */

char event_msg[] =
	"% was getting rather hungry.\000% was getting very hungry.\000% was starving!\000% was gett"
	"ing tired.\000% was getting sleepy.\000% was hit and killed!\000% was drowned in the water!"
	"\000% was burned in the lava.\000% was turned to stone by the witch.\000% started the journ"
	"ey in his home village of Tambry\000as had his brother before him.\000as had his brothers b"
	"efore him.\000% just couldn't stay awake any longer!\000% was feeling quite full.\000% was "
	"feeling quite rested.\000Even % would not be stupid enough to draw weapon in here.\000A gre"
	"at calming influence comes over %, preventing him from drawing his weapon.\000% picked up a"
	" scrap of paper.\000It read: \"Find the turtle!\"\000It read: \"Meet me at midnight at the "
	"Crypt. Signed, the Wraith Lord.\"\000% looked around but discovered nothing.\000% does not "
	"have that item.\000% bought some food and ate it.\000% bought some arrows.\000% passed out "
	"from hunger!\000% is not sleepy.\000% was tired, so he decided to lie down and sleep.\000% "
	"perished in the hot lava!\000It was midnight.\000It was morning.\000It was midday.\000Eveni"
	"ng was drawing near.\000Ground is too hot for swan to land.\000Flying too fast to dismount."
	"\000\"They're all dead!\" he cried.\000No time for that now!\000% put an apple away for lat"
	"er.\000% ate one of his apples.\000% discovered a hidden object.\000";

static const char q1[] =
	"To Quest for the...?\000";

static const char q2[] =
	"Make haste, but take...?\000";

static const char q3[] =
	"Scorn murderous...?\000";

static const char q4[] =
	"Summon the...?\000";

static const char q5[] =
	"Wing forth in...?\000";

static const char q6[] =
	"Hold fast to your...?\000";

static const char q7[] =
	"Defy Ye that...?\000";

static const char q8[] =
	"In black darker than...?\000";

unsigned char place_tbl[] = {
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
};

unsigned char inside_tbl[] = {
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
};

char place_msg[] =
	"\000\000% returned to the village of Tambry.\000% came to Vermillion Manor.\000% reached th"
	"e Mountains of Frost\000% reached the Plain of Grief.\000% came to the city of Marheim.\000"
	"% came to the Witch's castle.\000% came to the Graveyard.\000% came to a great stone ring.\000"
	"% came to a watchtower.\000% traveled to the great Bog.\000% came to the Crystal Palace.\000"
	"% came to mysterious Pixle Grove.\000% entered the Citadel of Doom.\000% entered the Burnin"
	"g Waste.\000% found an oasis.\000% came to the hidden city of Azal.\000% discovered an outl"
	"ying fort.\000% came to a small keep.\000% came to an old castle.\000% came to a log cabin."
	"\000% came to a dark stone tower.\000% came to an isolated cabin.\000% came to the Tombs of"
	" Hemsath.\000% reached the Forbidden Keep.\000% found a cave in the hillside.\000";

char inside_msg[] =
	"\000\000% came to a small chamber.\000% came to a large chamber.\000% came to a long passag"
	"eway.\000% came to a twisting tunnel.\000% came to a forked intersection.\000He entered the"
	" keep.\000He entered the castle.\000He entered the castle of King Mar.\000He entered the sa"
	"nctuary of the temple.\000% entered the Spirit Plane.\000% came to a large room.\000% came "
	"to an octagonal room.\000% traveled along a stone corridor.\000% came to a stone maze.\000H"
	"e entered a small building.\000He entered the building.\000He entered the tavern.\000He wen"
	"t inside the inn.\000He entered the crypt.\000He walked into the cabin.\000He unlocked the "
	"door and entered.\000";

static const char msg1[] =
	"\200\012\034   \"Rescue the Talisman!\"\200\012'was the Mayor's plea.\200\0122\"Only the Ta"
	"lisman can\200\012=protect our village from\200\012Hthe evil forces of the\200\012Snight.\""
	" And so Julian\200\012^set out on his quest to\200\012irecover it.\000";

static const char msg2[] =
	"\200\014,Unfortunately for Julian,\200\0147his luck had run out.\200\014BMany months passed"
	" and\200\014MJulian did not return...\000";

static const char msg3[] =
	"\200\024,So Phillip set out,\200\0247determined to find his\200\024Bbrother and complete\200"
	"\024Mthe quest.\000";

static const char msg4[] =
	"\200\014,But sadly, Phillip's\200\0147cleverness could not save\200\014Bhim from the same f"
	"ate\200\014Mas his older brother.\000";

static const char msg5[] =
	"\200\017\036So Kevin took up the\200\017)quest, risking all, for\200\0174the village had gr"
	"own\200\017?desperate. Young and\200\017Jinexperienced, his\200\017Uchances did not look\200"
	"\017`good.\000";

static const char msg6[] =
	"\200\012\037And so ends our sad tale.\200\014-The Lesson of the Story:\200!XStay at Home!\000"
	"";

static const char msg7[] =
	"\200\016 Having defeated the\200\016+villanous Necromancer\200\0166and recovered the\200\016"
	"ATalisman, \000";

static const char msg7a[] =
	"\200\016Lreturned to Marheim\200\016Wwhere he wed the\200\016bprincess...\000";

static const char msg8[] =
	"\200\012\032\000";

static const char msg8a[] =
	" had rescued Katra,\200\012%Princess of Marheim.\200\0120Though they had pledged\200\012;th"
	"eir love for each, \200\012Fother, \000";

static const char msg8b[] =
	" knew that\200\012Q his quest could not\200\012\\be forsaken.\000";

static const char msg9[] =
	"\200\012!\000";

static const char msg9a[] =
	" had rescued Karla\200\012,(Katra's sister), Princess\200\0127of Marheim. Though they\200\012"
	"Bhad pledged their love\200\012Mfor each other, \000";

static const char msg9b[] =
	"\200\012Xknew that his quest\200\012ccould not be forsaken.\000";

static const char msg10[] =
	"\200\012\032\000";

static const char msg10a[] =
	" had rescued Kandy\200\012%(Katra's and Karla's\200\0120sister), Princess of\200\012;Marhei"
	"m. Though they had\200\012Fpledged their love for\200\012Qeach other, \000";

static const char msg10b[] =
	" knew \200\012\\that his quest could\200\012gnot be forsaken.\000";

static const char msg11[] =
	"\200#%After seeing the\200\0210princess safely to her\200\021;home city, and with a\200\021"
	"Fking's gift in gold,\200\021Q\000";

static const char msg11a[] =
	" once more set\200\021\\out on his quest.\000";

static const char msg12[] =
	"\200@\023So...\200\021AYou, game seeker, would guide the\200\005Kbrothers to their destiny?"
	" You would\200\005Uaid them and give directions? Answer,\200\005_then, these three question"
	"s and prove\200\005iyour fitness to be their advisor:\000";

char speeches[] =
	"% attempted to communicate with the Ogre but a guttural snarl was the only response.\000\"H"
	"uman must die!\" said the goblin-man.\000\"Doom!\" wailed the wraith.\000A clattering of bo"
	"nes was the only reply.\000% knew that it is a waste of time to talk to a snake.\000...\000"
	"There was no reply.\000\"Die, foolish mortal!\" he said.\000\"No need to shout, son!\" he s"
	"aid.\000\"Nice weather we're having, isn't it?\" queried the ranger.\000\"Good luck, sonny!"
	"\" said the ranger. \"Hope you win!\"\000\"If you need to cross the lake\" said the ranger,"
	" \"There's a raft just north of here.\"\000\"Would you like to buy something?\" said the ta"
	"vern keeper. \"Or do you just need lodging for the night?\"\000\"Good Morning.\" said the t"
	"avern keeper. \"Hope you slept well.\"\000\"Have a drink!\" said the tavern keeper.\"\000\""
	"State your business!\" said the guard.\015\"My business is with the king.\" stated %, respe"
	"ctfully.\000\"Please, sir, rescue me from this horrible prison!\" pleaded the princess.\000"
	"\"I cannot help you, young man.\" said the king. \"My armies are decimated, and I fear that"
	" with the loss of my children, I have lost all hope.\"\000\"Here is a writ designating you "
	"as my official agent. Be sure and show this to the Priest before you leave Marheim.\000\"I'"
	"m afraid I cannot help you, young man. I already gave the golden statue to the other young "
	"man.\000\"If you could rescue the king's daughter,\" said Lord Trane, \"The King's courage "
	"would be restored.\"\000\"Sorry, I have no use for it.\"\000\"The dragon's cave is directly"
	" north of here.\" said the ranger.\"\000\"Alms! Alms for the poor!\"\000\"I have a prophecy"
	" for you, m'lord.\" said the beggar. \"You must seek two women, one Good, one Evil.\"\000\""
	"Lovely Jewels, glint in the night - give to us the gift of Sight!\" he said.\000\"Where is "
	"the hidden city? How can you find it when you cannot even see it?\" said the beggar.\000\"K"
	"ind deeds could gain thee a friend from the sea.\"\000\"Seek the place that is darker than "
	"night - There you shall find your goal in sight!\" said the wizard, cryptically.\000\"Like "
	"the eye itself, a crystal Orb can help to find things concealed.\"\000\"The Witch lives in "
	"the dim forest of Grimwood, where the very trees are warped to her will. Her gaze is Death!"
	"\"\000\"Only the light of the Sun can destroy the Witch's Evil.\"\000\"The maiden you seek "
	"lies imprisoned in an unreachable castle surrounded by unclimbable mountains.\"\000\"Tame t"
	"he golden beast and no mountain may deny you! But what rope could hold such a creature?\"\000"
	"\"Just what I needed!\" he said.\000\"Away with you, young ruffian!\" said the Wizard. \"Pe"
	"rhaps you can find some small animal to torment if that pleases you!\"\000\"You must seek y"
	"our enemy on the spirit plane. It is hazardous in the extreme. Space may twist, and time it"
	"self may run backwards!\"\000\"When you wish to travel quickly, seek the power of the Stone"
	"s.\" he said.\000\"Since you are brave of heart, I shall Heal all your wounds.\"\015Instant"
	"ly % felt much better.\000\"Ah! You have a writ from the king. Here is one of the golden st"
	"atues of Azal-Car-Ithil. Find all five and you'll find the vanishing city.\"\000\"Repent, S"
	"inner! Thou art an uncouth brute and I have no interest in your conversation!\"\000\"Ho the"
	"re, young traveler!\" said the black figure. \"None may enter the sacred shrine of the Peop"
	"le who came Before!\"\000\"Your prowess in battle is great.\" said the Knight of Dreams. \""
	"You have earned the right to enter and claim the prize.\"\000\"So this is the so-called Her"
	"o who has been sent to hinder my plans. Simply Pathetic. Well, try this, young Fool!\"\000%"
	" gasped. The Necromancer had been transformed into a normal man. All of his evil was gone.\000"
	"\"%.\" said the Sorceress. \"Welcome. Here is one of the five golden figurines you will nee"
	"d.\"\015\"Thank you.\" said %.\000\"Look into my eyes and Die!!\" hissed the witch.\015\"No"
	"t a chance!\" replied %\000The Spectre spoke. \"HE has usurped my place as lord of undead. "
	"Bring me bones of the ancient King and I'll help you destroy him.\"\000% gave him the ancie"
	"nt bones.\015\"Good! That spirit now rests quietly in my halls. Take this crystal shard.\"\000"
	"\"%...\" said the apparition. \"I am the ghost of your dead brother. Find my bones -- there"
	" you will find some things you need.\000% gave him some gold coins. \015\"Why, thank you, y"
	"oung sir!\"\000\"Sorry, but I have nothing to sell.\"\000\000\"The dragon's cave is east of"
	" here.\" said the ranger.\"\000\"The dragon's cave is west of here.\" said the ranger.\"\000"
	"\"The dragon's cave is south of here.\" said the ranger.\"\000\"Oh, thank you for saving my"
	" eggs, kind man!\" said the turtle. \"Take this seashell as a token of my gratitude.\"\000\""
	"Just hop on my back if you need a ride somewhere.\" said the turtle.\000\"Stupid fool, you "
	"can't hurt me with that!\"\000\"Your magic won't work here, fool!\"\000The Sunstone has mad"
	"e the witch vulnerable!\000";

char titletext[] =
	"\200\034!\"The Faery Tale Adventure\"\200\024OAnimation, Programming & Music\200LZby\2008eD"
	"avid Joiner\200\024\240Copyright 1986 MicroIllusions \000";

unsigned char hinor[256] = {
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
};

unsigned char hivar[256] = {
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
};

const char *const placard_msgs[] = {
	msg1,
	msg2,
	msg3,
	msg4,
	msg5,
	msg6,
	msg7,
	msg7a,
	msg8,
	msg8a,
	msg8b,
	msg9,
	msg9a,
	msg9b,
	msg10,
	msg10a,
	msg10b,
	msg11,
	msg11a,
	msg12,
};

const char *const question_msgs[] = {
	q1,
	q2,
	q3,
	q4,
	q5,
	q6,
	q7,
	q8,
};

