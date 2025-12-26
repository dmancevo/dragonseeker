"""Game constants and configuration."""

# Game settings
MIN_PLAYERS = 3
MAX_PLAYERS = 12

# Game state cleanup
GAME_TTL_SECONDS = 3600  # Games are cleaned up after 1 hour

# WebSocket settings
WEBSOCKET_PING_INTERVAL = 30  # Ping every 30 seconds

# Word pairs for the game
# Format: (villager_word, knight_word)
# Villagers get the first word, Knights get the similar but different second word
WORD_PAIRS = [
    # Animals
    ("tiger", "lion"),
    ("elephant", "mammoth"),
    ("giraffe", "llama"),
    ("penguin", "puffin"),
    ("dolphin", "whale"),
    ("butterfly", "moth"),
    ("kangaroo", "wallaby"),
    ("octopus", "squid"),
    ("peacock", "parrot"),
    ("hamster", "guinea pig"),
    ("eagle", "hawk"),
    ("rabbit", "hare"),
    ("monkey", "ape"),
    ("panda", "koala"),
    ("shark", "barracuda"),

    # Objects
    ("telescope", "binoculars"),
    ("umbrella", "parasol"),
    ("guitar", "ukulele"),
    ("camera", "camcorder"),
    ("keyboard", "piano"),
    ("bicycle", "tricycle"),
    ("lighthouse", "watchtower"),
    ("compass", "GPS"),
    ("mirror", "reflection"),
    ("clock", "watch"),
    ("scissors", "shears"),
    ("laptop", "tablet"),
    ("backpack", "suitcase"),
    ("wallet", "purse"),
    ("sunglasses", "goggles"),

    # Places
    ("museum", "gallery"),
    ("library", "bookstore"),
    ("restaurant", "cafe"),
    ("airport", "heliport"),
    ("hospital", "clinic"),
    ("stadium", "arena"),
    ("theater", "cinema"),
    ("beach", "shore"),
    ("mountain", "hill"),
    ("forest", "jungle"),
    ("desert", "wasteland"),
    ("volcano", "geyser"),
    ("waterfall", "fountain"),
    ("castle", "fortress"),
    ("temple", "shrine"),

    # Food & Drink
    ("pizza", "flatbread"),
    ("sushi", "sashimi"),
    ("chocolate", "cocoa"),
    ("coffee", "espresso"),
    ("sandwich", "panini"),
    ("burger", "slider"),
    ("pasta", "noodles"),
    ("icecream", "gelato"),
    ("pancake", "waffle"),
    ("muffin", "cupcake"),
    ("cookie", "biscuit"),
    ("salad", "coleslaw"),
    ("soup", "stew"),
    ("cheese", "yogurt"),
    ("bread", "toast"),

    # Activities
    ("swimming", "diving"),
    ("dancing", "ballet"),
    ("painting", "drawing"),
    ("reading", "studying"),
    ("cooking", "baking"),
    ("singing", "humming"),
    ("hiking", "trekking"),
    ("fishing", "angling"),
    ("camping", "glamping"),
    ("skiing", "snowboarding"),
    ("surfing", "paddleboarding"),
    ("juggling", "circus"),
    ("gardening", "landscaping"),
    ("knitting", "crocheting"),
    ("writing", "typing"),

    # Weather & Nature
    ("rainbow", "aurora"),
    ("thunder", "lightning"),
    ("sunrise", "sunset"),
    ("snowflake", "hailstone"),
    ("breeze", "wind"),
    ("hurricane", "typhoon"),
    ("tornado", "cyclone"),
    ("earthquake", "tremor"),
    ("meteor", "comet"),
    ("eclipse", "solstice"),

    # Transportation
    ("airplane", "jet"),
    ("submarine", "U-boat"),
    ("helicopter", "chopper"),
    ("spaceship", "rocket"),
    ("motorcycle", "scooter"),
    ("sailboat", "yacht"),
    ("train", "locomotive"),
    ("car", "automobile"),
    ("bus", "coach"),
    ("balloon", "blimp"),

    # Sports
    ("basketball", "netball"),
    ("soccer", "football"),
    ("tennis", "badminton"),
    ("baseball", "cricket"),
    ("volleyball", "handball"),
    ("bowling", "curling"),
    ("boxing", "wrestling"),
    ("archery", "darts"),
    ("fencing", "swordplay"),
    ("hockey", "lacrosse"),

    # Professions
    ("teacher", "professor"),
    ("doctor", "surgeon"),
    ("chef", "cook"),
    ("pilot", "aviator"),
    ("astronaut", "cosmonaut"),
    ("detective", "investigator"),
    ("firefighter", "paramedic"),
    ("scientist", "researcher"),
    ("artist", "painter"),
    ("musician", "composer"),

    # Household Items
    ("refrigerator", "freezer"),
    ("microwave", "oven"),
    ("toaster", "grill"),
    ("blender", "mixer"),
    ("vacuum", "broom"),
    ("television", "monitor"),
    ("pillow", "cushion"),
    ("blanket", "duvet"),
    ("curtain", "blinds"),
    ("lamp", "chandelier"),
]
