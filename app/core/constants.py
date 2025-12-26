"""Game constants and configuration."""

# Game settings
MIN_PLAYERS = 3
MAX_PLAYERS = 12

# Game state cleanup
GAME_TTL_SECONDS = 3600  # Games are cleaned up after 1 hour

# WebSocket settings
WEBSOCKET_PING_INTERVAL = 30  # Ping every 30 seconds

# Word list for the game
WORD_LIST = [
    # Animals
    "elephant", "giraffe", "penguin", "dolphin", "butterfly",
    "kangaroo", "octopus", "peacock", "flamingo", "hamster",
    "tiger", "eagle", "rabbit", "monkey", "panda",

    # Objects
    "telescope", "umbrella", "guitar", "camera", "keyboard",
    "bicycle", "lighthouse", "compass", "mirror", "clock",
    "scissors", "laptop", "backpack", "wallet", "sunglasses",

    # Places
    "museum", "library", "restaurant", "airport", "hospital",
    "stadium", "theater", "beach", "mountain", "forest",
    "desert", "volcano", "waterfall", "castle", "temple",

    # Food & Drink
    "pizza", "sushi", "chocolate", "coffee", "sandwich",
    "burger", "pasta", "icecream", "pancake", "muffin",
    "cookie", "salad", "soup", "cheese", "bread",

    # Activities
    "swimming", "dancing", "painting", "reading", "cooking",
    "singing", "hiking", "fishing", "camping", "skiing",
    "surfing", "juggling", "gardening", "knitting", "drawing",

    # Weather & Nature
    "rainbow", "thunder", "sunrise", "snowflake", "breeze",
    "hurricane", "tornado", "earthquake", "meteor", "eclipse",

    # Transportation
    "airplane", "submarine", "helicopter", "spaceship", "motorcycle",
    "sailboat", "train", "rocket", "scooter", "balloon",

    # Sports
    "basketball", "soccer", "tennis", "baseball", "volleyball",
    "bowling", "boxing", "archery", "fencing", "hockey",

    # Professions
    "teacher", "doctor", "chef", "pilot", "astronaut",
    "detective", "firefighter", "scientist", "artist", "musician",

    # Household Items
    "refrigerator", "microwave", "toaster", "blender", "vacuum",
    "television", "pillow", "blanket", "curtain", "lamp"
]
