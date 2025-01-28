use rand::prelude::IndexedRandom;
use rand::Rng;
use rand::SeedableRng;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;

const TEMPORAL_TERMS: &[&str] = &[
    // Time periods
    "Vespertine",
    "Matutinal",
    "Crepuscular",
    "Meridian",
    "Autumnal",
    "Vernal",
    "Hibernal",
    "Aestival",
    "Perpetual",
    "Ephemeral",
    "Eternal",
    "Fugitive",
    "Evanescent",
    "Sempiternal",
    "Momentary",
    "Transient",
    "Fleeting",
    "Temporal",
    "Diurnal",
    "Nocturnal",
    "Quotidian",
    "Circadian",
    "Seasonal",
    "Perennial",
    // Ancient periods
    "Archaic",
    "Antediluvian",
    "Prehistoric",
    "Primordial",
    "Primeval",
    "Pristine",
    "Ancient",
    "Antique",
    "Ancestral",
    "Aboriginal",
    "Primitive",
    "Primal",
    "Protean",
    "Atavistic",
    "Venerable",
    "Immemorial",
    "Eternal",
    "Ageless",
    "Timeless",
    "Classical",
    "Historic",
    "Mythic",
    "Legendary",
    "Traditional",
    // Time cycles
    "Cyclical",
    "Rhythmic",
    "Periodic",
    "Oscillating",
    "Undulating",
    "Wavelike",
    "Pulsating",
    "Recurring",
    "Repetitive",
    "Sequential",
    "Successive",
    "Consecutive",
    "Progressive",
    "Linear",
    "Circular",
    "Spiral",
    "Helical",
    "Orbital",
    "Rotational",
    "Revolutionary",
    "Cyclic",
    "Iterative",
    "Recursive",
    "Repeating",
    // Time flow
    "Flowing",
    "Streaming",
    "Drifting",
    "Passing",
    "Moving",
    "Running",
    "Elapsing",
    "Advancing",
    "Progressing",
    "Proceeding",
    "Continuing",
    "Ongoing",
    "Enduring",
    "Lasting",
    "Persisting",
    "Remaining",
    "Abiding",
    "Lingering",
    "Staying",
    "Dwelling",
    "Residing",
    "Existing",
    "Being",
    "Becoming",
    // Time qualities
    "Swift",
    "Rapid",
    "Quick",
    "Fast",
    "Speedy",
    "Hasty",
    "Slow",
    "Gradual",
    "Leisurely",
    "Unhurried",
    "Measured",
    "Deliberate",
    "Steady",
    "Regular",
    "Constant",
    "Fixed",
    "Stable",
    "Unchanging",
    "Variable",
    "Changing",
    "Fluctuating",
    "Varying",
    "Altering",
    "Shifting",
    // Time divisions
    "Moment",
    "Instant",
    "Second",
    "Minute",
    "Hour",
    "Day",
    "Week",
    "Month",
    "Season",
    "Quarter",
    "Year",
    "Decade",
    "Century",
    "Millennium",
    "Eon",
    "Era",
    "Age",
    "Epoch",
    "Period",
    "Phase",
    "Stage",
    "Interval",
    "Duration",
    "Span",
    // Celestial time
    "Solar",
    "Lunar",
    "Stellar",
    "Cosmic",
    "Galactic",
    "Universal",
    "Planetary",
    "Orbital",
    "Ecliptic",
    "Zodiacal",
    "Astronomical",
    "Celestial",
    "Sidereal",
    "Temporal",
    "Eternal",
    "Infinite",
    "Boundless",
    "Limitless",
    "Endless",
    "Immortal",
    "Perpetual",
    "Everlasting",
    "Undying",
    "Deathless",
    // Time perception
    "Present",
    "Past",
    "Future",
    "Now",
    "Then",
    "When",
    "Before",
    "After",
    "During",
    "While",
    "Meanwhile",
    "Interim",
    "Interval",
    "Period",
    "Phase",
    "Stage",
    "Era",
    "Epoch",
    "Age",
    "Time",
    "Season",
    "Cycle",
    "Round",
    "Revolution",
    // Time periods
    "Vespertine",
    "Matutinal",
    "Crepuscular",
    "Meridian",
    "Autumnal",
    "Vernal",
    "Hibernal",
    "Aestival",
    "Perpetual",
    "Ephemeral",
    "Eternal",
    "Fugitive",
    "Evanescent",
    "Sempiternal",
    "Momentary",
    "Transient",
    "Fleeting",
    "Temporal",
    "Diurnal",
    "Nocturnal",
    "Quotidian",
    "Circadian",
    "Seasonal",
    "Perennial",
    // Ancient periods
    "Archaic",
    "Antediluvian",
    "Prehistoric",
    "Primordial",
    "Primeval",
    "Pristine",
    "Ancient",
    "Antique",
    "Ancestral",
    "Aboriginal",
    "Primitive",
    "Primal",
    "Protean",
    "Atavistic",
    "Venerable",
    "Immemorial",
    "Eternal",
    "Ageless",
    "Timeless",
    "Classical",
    "Historic",
    "Mythic",
    "Legendary",
    "Traditional",
    // Time cycles
    "Cyclical",
    "Rhythmic",
    "Periodic",
    "Oscillating",
    "Undulating",
    "Wavelike",
    "Pulsating",
    "Recurring",
    "Repetitive",
    "Sequential",
    "Successive",
    "Consecutive",
    "Progressive",
    "Linear",
    "Circular",
    "Spiral",
    "Helical",
    "Orbital",
    "Rotational",
    "Revolutionary",
    "Cyclic",
    "Iterative",
    "Recursive",
    "Repeating",
    // Time flow
    "Flowing",
    "Streaming",
    "Drifting",
    "Passing",
    "Moving",
    "Running",
    "Elapsing",
    "Advancing",
    "Progressing",
    "Proceeding",
    "Continuing",
    "Ongoing",
    "Enduring",
    "Lasting",
    "Persisting",
    "Remaining",
    "Abiding",
    "Lingering",
    "Staying",
    "Dwelling",
    "Residing",
    "Existing",
    "Being",
    "Becoming",
    // Time qualities
    "Swift",
    "Rapid",
    "Quick",
    "Fast",
    "Speedy",
    "Hasty",
    "Slow",
    "Gradual",
    "Leisurely",
    "Unhurried",
    "Measured",
    "Deliberate",
    "Steady",
    "Regular",
    "Constant",
    "Fixed",
    "Stable",
    "Unchanging",
    "Variable",
    "Changing",
    "Fluctuating",
    "Varying",
    "Altering",
    "Shifting",
    // Philosophical time
    "Eternal",
    "Infinite",
    "Absolute",
    "Relative",
    "Subjective",
    "Objective",
    "Metaphysical",
    "Transcendent",
    "Immanent",
    "Universal",
    "Particular",
    "Essential",
    "Existential",
    "Phenomenal",
    "Noumenal",
    "Temporal",
    "Spiritual",
    "Material",
    "Concrete",
    "Abstract",
    "Real",
    "Ideal",
    "Actual",
    "Potential",
    // Mystical time
    "Ethereal",
    "Celestial",
    "Divine",
    "Sacred",
    "Holy",
    "Blessed",
    "Consecrated",
    "Hallowed",
    "Sanctified",
    "Venerated",
    "Revered",
    "Worshipped",
    "Mystical",
    "Magical",
    "Enchanted",
    "Supernatural",
    "Preternatural",
    "Supernal",
    "Otherworldly",
    "Transcendental",
    "Sublime",
    "Ineffable",
    "Mysterious",
    "Secret",
    // Quantum time
    "Quantum",
    "Relativistic",
    "Unified",
    "Entangled",
    "Superposed",
    "Coherent",
    "Decoherent",
    "Simultaneous",
    "Synchronous",
    "Concurrent",
    "Parallel",
    "Sequential",
    "Causal",
    "Acausal",
    "Deterministic",
    "Probabilistic",
    "Uncertain",
    "Indefinite",
    "Discrete",
    "Continuous",
    "Wave",
    "Particle",
    "Field",
    "String",
    // Geological time
    "Precambrian",
    "Paleozoic",
    "Mesozoic",
    "Cenozoic",
    "Hadean",
    "Archean",
    "Proterozoic",
    "Cambrian",
    "Ordovician",
    "Silurian",
    "Devonian",
    "Carboniferous",
    "Permian",
    "Triassic",
    "Jurassic",
    "Cretaceous",
    "Paleogene",
    "Neogene",
    "Quaternary",
    "Holocene",
    "Pleistocene",
    "Pliocene",
    "Miocene",
    "Oligocene",
    // Additional temporal qualities
    "Liminal",
    "Transitional",
    "Intermediate",
    "Interstitial",
    "Threshold",
    "Boundary",
    "Marginal",
    "Peripheral",
    "Central",
    "Focal",
    "Nodal",
    "Pivotal",
    "Critical",
    "Crucial",
    "Essential",
    "Fundamental",
    "Basic",
    "Primary",
    "Secondary",
    "Tertiary",
    "Quaternary",
    "Ultimate",
    "Penultimate",
    "Final",
];

// Rich vocabulary categorized by color associations and meaning
const LUMINOUS_TERMS: &[&str] = &[
    // Original luminous terms
    "Resplendent",
    "Effulgent",
    "Lustrous",
    "Radiant",
    "Phosphorescent",
    "Incandescent",
    "Refulgent",
    "Lambent",
    "Scintillating",
    "Coruscating",
    "Effervescent",
    "Luminescent",
    "Prismatic",
    "Crystalline",
    "Pellucid",
    "Translucent",
    "Diaphanous",
    "Ethereal",
    // Light quality terms
    "Gleaming",
    "Glistening",
    "Glowing",
    "Shimmering",
    "Sparkling",
    "Twinkling",
    "Brilliant",
    "Dazzling",
    "Flashing",
    "Flickering",
    "Glinting",
    "Glittering",
    "Illuminated",
    "Iridescent",
    "Lucent",
    "Luminous",
    "Opalescent",
    "Pearlescent",
    // Surface quality terms
    "Phosphoric",
    "Radiant",
    "Refulgent",
    "Scintillant",
    "Shining",
    "Silvery",
    "Sleek",
    "Glossy",
    "Burnished",
    "Polished",
    "Reflective",
    "Sheeny",
    "Bright",
    "Vivid",
    "Intense",
    "Burning",
    "Fierce",
    "Fulgent",
    // Scientific light terms
    "Glaring",
    "Blazing",
    "Beaming",
    "Efflorescent",
    "Lucid",
    "Luminiferous",
    "Nitid",
    "Pyrescent",
    "Rutilant",
    "Splendent",
    "Vibrant",
    "Vitric",
    "Volucent",
    "Argent",
    "Fulgurant",
    "Luminary",
    "Photonic",
    "Stellar",
    // Celestial terms
    "Astral",
    "Empyrean",
    "Celestial",
    "Solar",
    "Galactic",
    "Meteoric",
    "Nebular",
    "Phosphenic",
    "Plasmic",
    "Quasaric",
    "Spectral",
    "Luminal",
    "Auroral",
    "Candescent",
    "Fulminant",
    "Ignescent",
    "Luciferous",
    "Radiative",
    // Technical light terms
    "Refractile",
    "Selenian",
    "Strobic",
    "Voltaic",
    "Wavelike",
    "Xenogenic",
    "Halogenic",
    "Heliacal",
    "Irradiant",
    "Klieg",
    "Lucific",
    "Luminific",
    "Nitent",
    "Opulent",
    "Photogenic",
    "Prismal",
    "Radious",
    "Refractant",
    // Additional luminous qualities
    "Relucent",
    "Rutilated",
    "Scintillous",
    "Sidereal",
    "Solarious",
    "Spectrous",
    "Spherical",
    "Starlike",
    "Stellular",
    "Translucid",
    "Uranous",
    "Vitreous",
    "Vulcanic",
    "Xanthous",
    "Zenithal",
    "Zodiacal",
    "Zonal",
    "Phosphorific",
    // Modern technical terms
    "Bioluminescent",
    "Chemiluminescent",
    "Electroluminescent",
    "Fluorescent",
    "Phosphorescent",
    "Photoluminescent",
    "Radioluminescent",
    "Thermoluminescent",
    "Triboluminescent",
    "Crystalluminescent",
    // Expanded light quality terms
    "Aureate",
    "Candescent",
    "Effusive",
    "Fulgorous",
    "Illuminant",
    "Luciferous",
    "Luminiferous",
    "Nitescent",
    "Photoelectric",
    "Photogenic",
    "Phototrophic",
    "Pyrogenic",
    // Additional surface qualities
    "Adamantine",
    "Chatoyant",
    "Nacrous",
    "Specular",
    "Vitreous",
    "Metallic",
    "Satiny",
    "Silken",
    "Glazed",
    "Lacquered",
    "Lustrated",
    "Polished",
    // Light movement terms
    "Cascading",
    "Dancing",
    "Flowing",
    "Oscillating",
    "Pulsating",
    "Rippling",
    "Streaming",
    "Undulating",
    "Waving",
    "Weaving",
    "Flickering",
    "Fluttering",
    // Atmospheric light terms
    "Crepuscular",
    "Dawning",
    "Meridian",
    "Nocturnal",
    "Twilight",
    "Vespertine",
    "Matutinal",
    "Noonday",
    "Sunlit",
    "Moonlit",
    "Starlit",
    "Phosphoric",
    // Color-specific luminous terms
    "Argenteous",
    "Aureolin",
    "Chryseous",
    "Flavescent",
    "Gilded",
    "Aurific",
    "Chryselephantine",
    "Chrysographic",
    "Flavescent",
    "Luteous",
    "Orpiment",
    "Xanthic",
    // Enhanced surface qualities
    "Alabastrine",
    "Crystalloid",
    "Diamonds",
    "Gemlike",
    "Jeweled",
    "Lapideous",
    "Marmoreal",
    "Perlaceous",
    "Phosphoroid",
    "Spherulitic",
    "Stellar",
    "Vitric",
    // Light intensity terms
    "Blazing",
    "Blinding",
    "Brilliant",
    "Dazzling",
    "Fierce",
    "Flaming",
    "Flaring",
    "Flashing",
    "Fulgurant",
    "Glaring",
    "Intense",
    "Piercing",
    // Specialized scientific terms
    "Biophotonic",
    "Chromatic",
    "Diffractive",
    "Holographic",
    "Interferometric",
    "Laser",
    "Optical",
    "Photonic",
    "Plasmonic",
    "Quantum",
    "Spectral",
    "Waveguide",
    // Metaphysical light terms
    "Celestial",
    "Divine",
    "Empyreal",
    "Ethereal",
    "Heavenly",
    "Immortal",
    "Olympian",
    "Paradisiac",
    "Supernal",
    "Transcendent",
    "Uranological",
    "Zenith",
    // Modern technology terms
    "Cyberluminescent",
    "Digital",
    "Electronic",
    "Holographic",
    "Laser",
    "Neon",
    "Plasma",
    "Quantum",
    "Synthetic",
    "Virtual",
    "Xenon",
    "LED",
];

const DEEP_TERMS: &[&str] = &[
    "Abyssal",
    "Profound",
    "Fathomless",
    "Tenebrous",
    "Cimmerian",
    "Stygian",
    "Umbrageous",
    "Nocturnal",
    "Plutonic",
    "Primordial",
    "Ancient",
    "Eternal",
    "Shadowed",
    "Obscure",
    "Arcane",
    "Mysterious",
    "Enigmatic",
    "Occult",
    "Chthonic",
    "Nether",
    "Infernal",
    "Tartarean",
    "Hadean",
    "Erebean",
    "Darksome",
    "Dusky",
    "Gloaming",
    "Murky",
    "Obsidian",
    "Onyx",
    "Ebon",
    "Sable",
    "Jetty",
    "Pitch",
    "Inky",
    "Anthracite",
    "Chalcedonic",
    "Adamantine",
    "Basaltic",
    "Cavernous",
    "Subterranean",
    "Chthonian",
    "Eldritch",
    "Esoteric",
    "Hermetic",
    "Mystic",
    "Orphic",
    "Recondite",
    "Cryptic",
    "Covert",
    "Hidden",
    "Latent",
    "Veiled",
    "Shrouded",
    "Umbral",
    "Shaded",
    "Darkling",
    "Benighted",
    "Crepuscular",
    "Dimmed",
    "Lightless",
    "Unlit",
    "Rayless",
    "Sunless",
    "Starless",
    "Moonless",
    "Aphotic",
    "Caliginous",
    "Fuliginous",
    "Murky",
    "Nebulous",
    "Nubilous",
    "Obfuscated",
    "Occulted",
    "Opaque",
    "Overcast",
    "Shadowy",
    "Subfusc",
    "Tenebrific",
    "Tenebrious",
    "Tenebruous",
    "Umbratile",
    "Umbriferous",
    "Umbrific",
    "Stygian",
    "Acherontic",
    "Avernian",
    "Cocytean",
    "Lethean",
    "Phlegethontic",
    "Plutonian",
    "Spectral",
    "Phantasmal",
    "Ghostly",
    "Wraithlike",
    "Ethereal",
    "Nychthemeral",
    "Nyctotropic",
    "Noctivagant",
    "Vespertine",
    "Crepuscular",
    "Obscurant",
    "Mysterious",
    "Ineffable",
    "Inscrutable",
    "Inexplicable",
    "Unfathomable",
    "Impenetrable",
    "Profound",
    "Bottomless",
    "Measureless",
    "Soundless",
    "Unlimited",
    "Boundless",
    "Infinite",
    "Endless",
    "Eternal",
    "Perpetual",
    "Sempiternal",
    "Everlasting",
    "Immemorial",
    "Ageless",
    "Timeless",
    "Dateless",
    "Primeval",
    "Pristine",
    "Archaic",
    "Antediluvian",
    "Prehistoric",
    "Primal",
    "Primitive",
    "Protogenic",
];

const NATURAL_ELEMENTS: &[&str] = &[
    "Obsidian",
    "Alabaster",
    "Jade",
    "Amber",
    "Onyx",
    "Malachite",
    "Azurite",
    "Beryl",
    "Chalcedony",
    "Carnelian",
    "Citrine",
    "Jasper",
    "Lapis",
    "Moonstone",
    "Opal",
    "Peridot",
    "Sapphire",
    "Topaz",
    "Zircon",
    "Tourmaline",
    "Adamantine",
    "Agate",
    "Alexandrite",
    "Almandine",
    "Amazonite",
    "Amethyst",
    "Andalusite",
    "Anglesite",
    "Apatite",
    "Aquamarine",
    "Aventurine",
    "Axinite",
    "Azurite",
    "Benitoite",
    "Beryllonite",
    "Bloodstone",
    "Boleite",
    "Bronzite",
    "Cairngorm",
    "Calcite",
    "Cassiterite",
    "Celestine",
    "Cerussite",
    "Chrysoberyl",
    "Chrysocolla",
    "Chrysoprase",
    "Cinnabar",
    "Cordierite",
    "Corundum",
    "Cryolite",
    "Cuprite",
    "Danburite",
    "Diamond",
    "Diaspore",
    "Diopside",
    "Dioptase",
    "Dolomite",
    "Dumortierite",
    "Emerald",
    "Enstatite",
    "Epidote",
    "Euclase",
    "Feldspar",
    "Fluorite",
    "Forsterite",
    "Gahnite",
    "Garnet",
    "Goshenite",
    "Grandidierite",
    "Grossular",
    "Hauyne",
    "Heliodor",
    "Hematite",
    "Hemimorphite",
    "Hessonite",
    "Hiddenite",
    "Howlite",
    "Iolite",
    "Ivory",
    "Jadeite",
    "Jeremejevite",
    "Kornerupine",
    "Kunzite",
    "Kyanite",
    "Labradorite",
    "Larimar",
    "Lazulite",
    "Lepidolite",
    "Leucite",
    "Morganite",
    "Musgravite",
    "Nephrite",
    "Oligoclase",
    "Olivine",
    "Painite",
    "Pargasite",
    "Pectolite",
    "Periclase",
    "Phenakite",
    "Phosphophyllite",
    "Prehnite",
    "Pyrope",
    "Quartz",
    "Rhodochrosite",
    "Rhodonite",
    "Rubelite",
    "Ruby",
    "Rutile",
    "Sanidine",
    "Scapolite",
    "Scheelite",
    "Scolecite",
    "Serpentine",
    "Serandite",
    "Smithsonite",
    "Sodalite",
    "Spessartine",
    "Sphalerite",
    "Sphene",
    "Spinel",
    "Spodumene",
    "Staurolite",
    "Stolzite",
    "Sugilite",
    "Sunstone",
    "Tanzanite",
    "Taaffeite",
    "Thomsonite",
    "Thumalite",
    "Tsavorite",
    "Tugtupite",
    "Turquoise",
    "Ulexite",
    "Uvarovite",
    "Variscite",
    "Vesuvianite",
    "Vivianite",
    "Wavellite",
    "Willemite",
    "Wurtzite",
    "Xenotime",
    "Zoisite",
    "Adularia",
    "Aegirine",
    "Analcime",
    "Anatase",
];

const BOTANICAL_TERMS: &[&str] = &[
    // Classical flowers
    "Amaranth",
    "Chrysanthemum",
    "Dahlia",
    "Edelweiss",
    "Foxglove",
    "Gardenia",
    "Heliotrope",
    "Iris",
    "Jasmine",
    "Kerria",
    "Lilac",
    "Magnolia",
    "Narcissus",
    "Orchid",
    "Peony",
    "Quince",
    "Rose",
    "Snapdragon",
    "Tulip",
    "Verbena",
    // Exotic flowers
    "Alstroemeria",
    "Begonia",
    "Calathea",
    "Dendrobium",
    "Euphorbia",
    "Frangipani",
    "Ginger",
    "Heliconia",
    "Ixora",
    "Jacaranda",
    "Kalanchoe",
    "Lobelia",
    "Mimosa",
    "Nerine",
    "Oleander",
    "Plumeria",
    "Quesnelia",
    "Rafflesia",
    "Strelitzia",
    "Tillandsia",
    // Trees
    "Acacia",
    "Birch",
    "Cedar",
    "Dogwood",
    "Elm",
    "Fir",
    "Ginkgo",
    "Hawthorn",
    "Ironwood",
    "Juniper",
    "Katsura",
    "Larch",
    "Maple",
    "Neem",
    "Oak",
    "Pine",
    "Quercus",
    "Redwood",
    "Spruce",
    "Teak",
    // Herbs
    "Anise",
    "Basil",
    "Chamomile",
    "Dill",
    "Echinacea",
    "Fennel",
    "Ginseng",
    "Horehound",
    "Ivy",
    "Juniper",
    "Kava",
    "Lavender",
    "Mint",
    "Nettle",
    "Oregano",
    "Parsley",
    "Quillja",
    "Rosemary",
    "Sage",
    "Thyme",
    // Fruits
    "Apricot",
    "Blackberry",
    "Cherry",
    "Damson",
    "Elderberry",
    "Fig",
    "Grape",
    "Hawthorn",
    "Indian Plum",
    "Jujube",
    "Kiwi",
    "Lemon",
    "Mango",
    "Nectarine",
    "Orange",
    "Peach",
    "Quince",
    "Raspberry",
    "Strawberry",
    "Tangerine",
    // Vines
    "Aristolochia",
    "Bougainvillea",
    "Clematis",
    "Dipladenia",
    "Euonymus",
    "Ficus",
    "Glycine",
    "Honeysuckle",
    "Ipomoea",
    "Jasmine",
    "Kudzu",
    "Lonicera",
    "Mandevilla",
    "Nasturtium",
    "Oncidium",
    "Passiflora",
    "Quisqualis",
    "Roxburghia",
    // Leaves
    "Acanthus",
    "Begonia",
    "Calathea",
    "Dracaena",
    "Eucalyptus",
    "Fern",
    "Geranium",
    "Hosta",
    "Ivy",
    "Jade",
    "Kale",
    "Lantana",
    "Monstera",
    "Nephthytis",
    "Oak",
    "Palmetto",
    "Quercus",
    "Rubus",
    "Sage",
    "Thalia",
    // Forest plants
    "Agarwood",
    "Bracken",
    "Clubmoss",
    "Dryad",
    "Elderwood",
    "Fernbrake",
    "Groundsel",
    "Heathland",
    "Ivywort",
    "Jewelweed",
    "Kinnikinnick",
    "Lichen",
    "Meadowsweet",
    "Nightshade",
    "Oldwood",
    "Pinedrop",
    "Queenofthemeadow",
    "Rushweed",
    "Sorrel",
    // Aquatic plants
    "Algae",
    "Bladderwort",
    "Cattail",
    "Duckweed",
    "Eelgrass",
    "Floatingheart",
    "Grasswrack",
    "Hydrilla",
    "Iris",
    "Juncus",
    "Kelp",
    "Lotus",
    "Myriophyllum",
    "Nymphaea",
    "Pondweed",
    "Quillwort",
    "Reedmace",
    "Sagittaria",
    "Tapegrass",
    // Mosses and lichens
    "Acrocarp",
    "Bogmoss",
    "Cushionmoss",
    "Deermoss",
    "Earthmoss",
    "Feathermoss",
    "Goldenmoss",
    "Haircap",
    "Icemoss",
    "Junipermoss",
    "Knot moss",
    "Leafymoss",
    "Maplemoss",
    "Oakmoss",
    "Pleurocarp",
    "Quaking moss",
    "Rockmoss",
    "Silvermoss",
    // Additional terms
    "Aril",
    "Bract",
    "Calyx",
    "Drupe",
    "Endocarp",
    "Frond",
    "Glume",
    "Hilum",
    "Inflorescence",
    "Juvenile",
    "Keel",
    "Lamina",
    "Meristem",
    "Node",
    "Ovule",
    "Petiole",
    "Quiescent",
    "Rachis",
    "Stipule",
    "Trichome",
];

const ATMOSPHERIC_TERMS: &[&str] = &[
    // Sky phenomena
    "Aurora",
    "Airglow",
    "Borealis",
    "Corona",
    "Crepuscular",
    "Dawn",
    "Eclipse",
    "Firmament",
    "Gloaming",
    "Horizon",
    "Ionosphere",
    "Jetstream",
    "Kelvin-Helmholtz",
    "Luminescence",
    "Meridian",
    "Nacreous",
    "Ozone",
    "Parallax",
    "Quaternary",
    "Radiance",
    // Cloud types
    "Altocumulus",
    "Bismuth",
    "Cirrus",
    "Drifting",
    "Ephemeral",
    "Fractus",
    "Gossamer",
    "Hovering",
    "Incus",
    "Jellyfish",
    "Kelvin",
    "Lenticular",
    "Mammatus",
    "Nimbus",
    "Opaque",
    "Pileus",
    "Quilted",
    "Radiatus",
    "Stratiform",
    "Towering",
    // Weather patterns
    "Anticyclone",
    "Breeze",
    "Cyclone",
    "Downburst",
    "Easterly",
    "Front",
    "Gale",
    "Hurricane",
    "Isobar",
    "Jet",
    "Katabatic",
    "Lightning",
    "Monsoon",
    "Northerly",
    "Occluded",
    "Pressure",
    "Quasi-stationary",
    "Ridge",
    "Storm",
    "Tempest",
    // Precipitation
    "Aerosol",
    "Blizzard",
    "Condensation",
    "Drizzle",
    "Evaporation",
    "Fog",
    "Graupel",
    "Hail",
    "Ice",
    "Januae",
    "Kinetic",
    "Liquid",
    "Mist",
    "Nebula",
    "Overcast",
    "Precipitation",
    "Quicksilver",
    "Rain",
    "Sleet",
    "Thunder",
    // Wind patterns
    "Aeolian",
    "Bora",
    "Chinook",
    "Derecho",
    "Etesian",
    "Foehn",
    "Gradient",
    "Harmattan",
    "Inversion",
    "Jetty",
    "Khamsin",
    "Levanter",
    "Mistral",
    "Norther",
    "Ostria",
    "Pampero",
    "Quasar",
    "Roaring",
    "Sirocco",
    "Trade",
    // Light phenomena
    "Afterglow",
    "Beam",
    "Caustic",
    "Diffraction",
    "Extinction",
    "Fluorescence",
    "Glare",
    "Halo",
    "Iridescence",
    "Jovian",
    "Kindle",
    "Luminous",
    "Mirage",
    "Noctilucent",
    "Optical",
    "Phosphorescence",
    "Quantum",
    "Refraction",
    "Scintillation",
    "Twilight",
    // Atmospheric layers
    "Aerosphere",
    "Boundary",
    "Chemosphere",
    "Dynamo",
    "Exosphere",
    "Fieldline",
    "Geocorona",
    "Heterosphere",
    "Ionosphere",
    "Jacobian",
    "Kenelly",
    "Lithosphere",
    "Mesosphere",
    "Neurosphere",
    "Ozonosphere",
    "Plasmasphere",
    "Quantum",
    "Radiosphere",
    "Stratosphere",
    "Thermosphere",
    // Space weather
    "Asteroid",
    "Binary",
    "Comet",
    "Debris",
    "Emission",
    "Flare",
    "Gamma",
    "Heliosphere",
    "Interstellar",
    "Jovian",
    "Kepler",
    "Lunar",
    "Meteor",
    "Nebula",
    "Orbital",
    "Pulsar",
    "Quasar",
    "Radio",
    "Solar",
    "Terrestrial",
    // Time of day
    "Alpen",
    "Breaklight",
    "Candescent",
    "Daybreak",
    "Evening",
    "Fading",
    "Gathering",
    "Halflight",
    "Interim",
    "Journey",
    "Kindling",
    "Lowering",
    "Morning",
    "Nightfall",
    "Onset",
    "Parting",
    "Quietus",
    "Rising",
    "Setting",
    "Terminal",
    // Additional terms
    "Absolute",
    "Baric",
    "Convective",
    "Diurnal",
    "Ephemeral",
    "Frontal",
    "Gradient",
    "Helical",
    "Inertial",
    "Kinematic",
    "Laminar",
    "Modal",
    "Nodal",
    "Orbital",
    "Periodic",
    "Quantum",
    "Radial",
    "Spiral",
    "Thermal",
    "Vortical",
    // Seasonal terms
    "Autumnal",
    "Brumal",
    "Cyclical",
    "Dormant",
    "Equinox",
    "Frigid",
    "Glacial",
    "Hibernal",
    "Icy",
    "Jovial",
    "Kalendar",
    "Liminal",
    "Meridional",
    "Nordic",
    "Occidental",
    "Polar",
    "Quarterly",
    "Seasonal",
    "Temporal",
    "Umbral",
];

const EMOTIONAL_TERMS: &[&str] = &[
    "Melancholic",
    "Euphoric",
    "Pensive",
    "Serene",
    "Contemplative",
    "Ecstatic",
    "Jubilant",
    "Wistful",
    "Nostalgic",
    "Tranquil",
    "Ethereal",
    "Sublime",
    "Ineffable",
    "Enigmatic",
    "Mysterious",
    "Phantasmal",
];

// Color family descriptors
const WARM_TERMS: &[&str] = &[
    // Traditional warm colors
    "Aureate",
    "Fulvous",
    "Ochre",
    "Russet",
    "Tawny",
    "Vermillion",
    "Carmine",
    "Sanguine",
    "Rufous",
    "Puce",
    "Cerise",
    "Scarlet",
    "Incarnadine",
    "Madder",
    // Red variations
    "Amaranth",
    "Cardinal",
    "Crimson",
    "Garnet",
    "Ruby",
    "Burgundy",
    "Cherry",
    "Cinnabar",
    "Coral",
    "Flame",
    "Maroon",
    "Rose",
    "Wine",
    "Blood",
    "Carnelian",
    "Copper",
    "Phoenix",
    "Rust",
    "Terracotta",
    "Vermilion",
    // Orange variations
    "Amber",
    "Apricot",
    "Cantaloupe",
    "Carrot",
    "Citrine",
    "Ginger",
    "Marigold",
    "Peach",
    "Persimmon",
    "Pumpkin",
    "Tangerine",
    "Sunset",
    "Honey",
    "Bronze",
    "Cinnamon",
    "Cognac",
    "Copper",
    "Rust",
    "Sepia",
    "Sienna",
    // Yellow variations
    "Aureolin",
    "Butter",
    "Canary",
    "Corn",
    "Daffodil",
    "Dandelion",
    "Gold",
    "Jasmine",
    "Lemon",
    "Maize",
    "Saffron",
    "Solar",
    "Sunshine",
    "Topaz",
    "Wheat",
    "Brass",
    "Desert",
    "Sand",
    "Straw",
    "Vanilla",
    // Brown variations
    "Auburn",
    "Beaver",
    "Brunette",
    "Burnt",
    "Caramel",
    "Chestnut",
    "Chocolate",
    "Coffee",
    "Dun",
    "Fawn",
    "Hazel",
    "Mahogany",
    "Nut",
    "Peanut",
    "Sepia",
    "Sorrel",
    "Tan",
    "Toast",
    "Umber",
    "Walnut",
    // Pink variations
    "Blush",
    "Carnation",
    "Champagne",
    "Flamingo",
    "Fuchsia",
    "Magenta",
    "Orchid",
    "Pearl",
    "Punch",
    "Raspberry",
    "Rouge",
    "Salmon",
    "Shell",
    "Tea",
    "Rose",
    "Watermelon",
    "Coral",
    "Peony",
    "Poppy",
    "Tulip",
    // Fire-related terms
    "Ablaze",
    "Burning",
    "Ember",
    "Fiery",
    "Flaming",
    "Igneous",
    "Inferno",
    "Kindle",
    "Molten",
    "Phoenix",
    "Pyre",
    "Scorched",
    "Smolder",
    "Torch",
    "Volcanic",
    "Wildfire",
    "Combustion",
    "Furnace",
    "Hearth",
    "Lava",
    // Sunset terms
    "Afterglow",
    "Crepuscular",
    "Dawn",
    "Daybreak",
    "Dusk",
    "Evening",
    "Horizon",
    "Morning",
    "Sunrise",
    "Sunset",
    "Twilight",
    "Vesper",
    "Golden",
    "Hour",
    "Meridian",
    "Noon",
    "Solar",
    "Sunlit",
    "Zenith",
    "Gloaming",
    // Spice terms
    "Allspice",
    "Cayenne",
    "Chili",
    "Cinnamon",
    "Curry",
    "Ginger",
    "Nutmeg",
    "Paprika",
    "Pepper",
    "Saffron",
    "Sage",
    "Spice",
    "Turmeric",
    "Wasabi",
    "Cardamom",
    "Cumin",
    "Mustard",
    "Pepper",
    "Sumac",
    "Tandoori",
    // Autumn terms
    "Autumn",
    "Fall",
    "Harvest",
    "Indian",
    "Maple",
    "October",
    "Deciduous",
    "Fallen",
    "Leafy",
    "Withered",
    "Bramble",
    "Bracken",
    "Thicket",
    "Woods",
    "Forest",
    "Grove",
    "Orchard",
    "Woodland",
    "Rustic",
    "Rural",
    // Desert terms
    "Arid",
    "Barren",
    "Canyon",
    "Desert",
    "Dune",
    "Mesa",
    "Plateau",
    "Sahara",
    "Sand",
    "Sandstone",
    "Sediment",
    "Stone",
    "Wasteland",
    "Bedrock",
    "Cliff",
    "Mountain",
    "Ridge",
    "Rock",
    "Valley",
    "Vista",
    // Additional warm terms
    "Antique",
    "Brass",
    "Bronze",
    "Burnished",
    "Copper",
    "Gilt",
    "Golden",
    "Metallic",
    "Patina",
    "Rustic",
    "Tarnished",
    "Vintage",
    "Weathered",
    "Aged",
    "Classic",
    "Historic",
    "Legacy",
    "Timeless",
    "Traditional",
    "Venerable",
];

const COOL_TERMS: &[&str] = &[
    // Classical blue terms
    "Cerulean",
    "Azure",
    "Ultramarine",
    "Cobalt",
    "Sapphire",
    "Indigo",
    "Navy",
    "Turquoise",
    "Aquamarine",
    "Beryl",
    "Cyan",
    "Teal",
    "Marine",
    "Ocean",
    "Aegean",
    "Lapis",
    "Arctic",
    "Glacial",
    "Frost",
    "Ice",
    // Green variations
    "Viridian",
    "Verdant",
    "Emerald",
    "Jade",
    "Malachite",
    "Sage",
    "Forest",
    "Chartreuse",
    "Olive",
    "Pine",
    "Moss",
    "Fern",
    "Seafoam",
    "Mint",
    "Laurel",
    "Ivy",
    "Spruce",
    "Eucalyptus",
    "Juniper",
    "Evergreen",
    // Purple variations
    "Amethyst",
    "Violet",
    "Lavender",
    "Lilac",
    "Mauve",
    "Plum",
    "Grape",
    "Heliotrope",
    "Periwinkle",
    "Thistle",
    "Orchid",
    "Mulberry",
    "Wine",
    "Royal",
    "Imperial",
    "Regal",
    "Noble",
    "Sovereign",
    "Palatial",
    "Magisterial",
    // Ocean terms
    "Abyssal",
    "Aqueous",
    "Marine",
    "Maritime",
    "Nautical",
    "Oceanic",
    "Pelagic",
    "Thalassic",
    "Wave",
    "Reef",
    "Coral",
    "Kelp",
    "Seaweed",
    "Depth",
    "Current",
    "Tide",
    "Flow",
    "Surge",
    "Swell",
    "Deep",
    // Ice terms
    "Glacial",
    "Arctic",
    "Polar",
    "Boreal",
    "Frigid",
    "Frozen",
    "Gelid",
    "Hyperborean",
    "Icy",
    "Nordic",
    "Septentrional",
    "Algid",
    "Brumal",
    "Hibernal",
    "Hiemal",
    "Wintry",
    "Crystalline",
    "Frost",
    "Rime",
    "Hoar",
    // Mineral terms
    "Adamantine",
    "Alabaster",
    "Beryl",
    "Crystal",
    "Diamond",
    "Gemstone",
    "Mineral",
    "Opal",
    "Pearl",
    "Quartz",
    "Crystalline",
    "Jeweled",
    "Precious",
    "Stone",
    "Zircon",
    "Aquamarine",
    "Topaz",
    "Moonstone",
    "Chalcedony",
    "Tourmaline",
    // Night terms
    "Nocturnal",
    "Crepuscular",
    "Vespertine",
    "Twilight",
    "Dusk",
    "Evening",
    "Midnight",
    "Starlit",
    "Moonlit",
    "Shadowy",
    "Tenebrous",
    "Dark",
    "Dim",
    "Gloaming",
    "Nightfall",
    "Umbra",
    "Penumbra",
    "Eclipse",
    "Stellar",
    "Cosmic",
    // Storm terms
    "Tempest",
    "Thunder",
    "Lightning",
    "Storm",
    "Squall",
    "Gale",
    "Whirlwind",
    "Cyclone",
    "Hurricane",
    "Typhoon",
    "Maelstrom",
    "Vortex",
    "Eddy",
    "Current",
    "Surge",
    "Wave",
    "Billow",
    "Breaker",
    "Swell",
    "Foam",
    // Mountain terms
    "Alpine",
    "Montane",
    "Highland",
    "Peak",
    "Summit",
    "Crest",
    "Ridge",
    "Pinnacle",
    "Apex",
    "Zenith",
    "Crown",
    "Top",
    "Spire",
    "Tower",
    "Height",
    "Elevation",
    "Altitude",
    "Ascent",
    "Climb",
    "Rise",
    // Cave terms
    "Cavern",
    "Grotto",
    "Cave",
    "Hollow",
    "Chamber",
    "Vault",
    "Crypt",
    "Catacomb",
    "Tunnel",
    "Passage",
    "Gallery",
    "Hall",
    "Corridor",
    "Channel",
    "Duct",
    "Conduit",
    "Path",
    "Way",
    "Route",
    "Track",
    // Forest terms
    "Sylvan",
    "Woodland",
    "Forest",
    "Grove",
    "Copse",
    "Thicket",
    "Wood",
    "Timber",
    "Branch",
    "Bough",
    "Leaf",
    "Needle",
    "Frond",
    "Blade",
    "Shoot",
    "Sprout",
    "Growth",
    "Green",
    "Verdant",
    "Lush",
    // Additional cool terms
    "Halcyon",
    "Serene",
    "Placid",
    "Tranquil",
    "Calm",
    "Peaceful",
    "Quiet",
    "Still",
    "Gentle",
    "Soft",
    "Mild",
    "Moderate",
    "Temperate",
    "Balanced",
    "Harmonious",
    "Unified",
    "Integrated",
    "Whole",
    "Complete",
    "Perfect",
];

const NEUTRAL_TERMS: &[&str] = &[
    // Gray variations
    "Taupe",
    "Ecru",
    "Beige",
    "Fawn",
    "Khaki",
    "Dove",
    "Cinereous",
    "Puce",
    "Buff",
    "Dun",
    "Feldgrau",
    "Greige",
    "Chamoisee",
    "Stone",
    "Slate",
    "Pewter",
    "Silver",
    "Ash",
    "Fog",
    "Mist",
    // Earth tones
    "Umber",
    "Sepia",
    "Sienna",
    "Terra",
    "Clay",
    "Loam",
    "Soil",
    "Sand",
    "Dust",
    "Dirt",
    "Ground",
    "Earth",
    "Mud",
    "Silt",
    "Sediment",
    "Deposit",
    "Layer",
    "Stratum",
    "Mineral",
    "Rock",
    // Metal terms
    "Platinum",
    "Steel",
    "Iron",
    "Nickel",
    "Chrome",
    "Tin",
    "Lead",
    "Zinc",
    "Aluminum",
    "Titanium",
    "Metallic",
    "Alloy",
    "Amalgam",
    "Ore",
    "Mineral",
    "Element",
    "Metal",
    "Material",
    "Substance",
    "Matter",
    // Stone terms
    "Granite",
    "Marble",
    "Slate",
    "Limestone",
    "Sandstone",
    "Quartz",
    "Flint",
    "Shale",
    "Basalt",
    "Gneiss",
    "Schist",
    "Feldspar",
    "Mica",
    "Chalk",
    "Pumice",
    "Tuff",
    "Travertine",
    "Dolomite",
    "Gypsum",
    "Alabaster",
    // Wood terms
    "Birch",
    "Ash",
    "Oak",
    "Maple",
    "Pine",
    "Cedar",
    "Elm",
    "Beech",
    "Poplar",
    "Willow",
    "Walnut",
    "Cherry",
    "Mahogany",
    "Teak",
    "Bamboo",
    "Reed",
    "Cane",
    "Cork",
    "Bark",
    "Grain",
    // Bone terms
    "Ivory",
    "Pearl",
    "Shell",
    "Bone",
    "Fossil",
    "Relic",
    "Remains",
    "Vestige",
    "Trace",
    "Fragment",
    "Piece",
    "Part",
    "Section",
    "Segment",
    "Division",
    "Portion",
    "Share",
    "Bit",
    "Morsel",
    "Crumb",
    // Paper terms
    "Parchment",
    "Vellum",
    "Paper",
    "Card",
    "Board",
    "Sheet",
    "Leaf",
    "Page",
    "Folio",
    "Scroll",
    "Roll",
    "Strip",
    "Band",
    "Ribbon",
    "Tape",
    "Film",
    "Layer",
    "Coating",
    "Cover",
    "Surface",
    // Textile terms
    "Linen",
    "Cotton",
    "Wool",
    "Silk",
    "Canvas",
    "Cloth",
    "Fabric",
    "Material",
    "Textile",
    "Weave",
    "Knit",
    "Mesh",
    "Net",
    "Web",
    "Lace",
    "Thread",
    "String",
    "Cord",
    "Rope",
    "Line",
    // Shadow terms
    "Shadow",
    "Shade",
    "Umbra",
    "Penumbra",
    "Dark",
    "Dim",
    "Dusk",
    "Twilight",
    "Evening",
    "Night",
    "Gloom",
    "Murk",
    "Obscurity",
    "Darkness",
    "Black",
    "Gray",
    "Charcoal",
    "Coal",
    "Soot",
    "Ink",
    // Light terms
    "White",
    "Bright",
    "Light",
    "Clear",
    "Pure",
    "Clean",
    "Fresh",
    "New",
    "Young",
    "Tender",
    "Soft",
    "Gentle",
    "Mild",
    "Sweet",
    "Kind",
    "Good",
    "Fair",
    "Fine",
    "Nice",
    "Pleasant",
    // Additional neutral terms
    "Moderate",
    "Medium",
    "Middle",
    "Central",
    "Normal",
    "Standard",
    "Regular",
    "Ordinary",
    "Common",
    "Usual",
    "Typical",
    "Average",
    "Mean",
    "Median",
    "Mode",
    "Norm",
    "Rule",
    "Way",
    "Custom",
    "Practice",
    // Time-worn terms
    "Aged",
    "Ancient",
    "Antique",
    "Archaic",
    "Classic",
    "Elderly",
    "Historic",
    "Mature",
    "Old",
    "Senior",
    "Venerable",
    "Vintage",
    "Weathered",
    "Worn",
    "Used",
    "Tired",
    "Weary",
    "Faded",
    "Dim",
    "Dull",
];

// [All the constant word arrays (LUMINOUS_TERMS, DEEP_TERMS, etc.) go here]
// (I've omitted them for brevity since we already have them)

#[derive(Serialize, Deserialize, Default)]
struct ColorMaps {
    hex_to_name: HashMap<String, String>,
    name_to_hex: HashMap<String, String>,
}

impl ColorMaps {
    fn add_color(&mut self, hex: &str, name: &str) {
        self.hex_to_name.insert(hex.to_string(), name.to_string());
        self.name_to_hex.insert(name.to_string(), hex.to_string());
    }

    fn name_exists(&self, name: &str) -> bool {
        self.name_to_hex.contains_key(name)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input_mermaid_file>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let content = fs::read_to_string(input_path)?;

    let colors = extract_colors(&content);
    if colors.is_empty() {
        println!("No (non-white) colors found in the file.");
        return Ok(());
    }

    let db_file_path = "colors.json";
    let mut color_db = load_color_db(db_file_path)?;

    let mut color_and_name = Vec::new();
    for hex in &colors {
        let name = get_or_create_artistic_name(hex, &mut color_db);
        color_and_name.push((hex.clone(), name));
    }

    save_color_db(db_file_path, &color_db)?;

    let palette_mermaid = generate_mermaid(color_and_name);

    let output_path = Path::new(input_path).with_extension("palette.mermaid");
    fs::write(&output_path, palette_mermaid)?;

    println!(
        "Palette generated with {} colors (excluding white).",
        colors.len()
    );
    println!("Saved to: {}", output_path.display());
    println!("Color names persisted to: {}", db_file_path);

    Ok(())
}

fn get_or_create_artistic_name(hex: &str, db: &mut ColorMaps) -> String {
    if let Some(existing) = db.hex_to_name.get(hex) {
        return existing.clone();
    }

    let new_name = generate_artistic_name(hex, db);
    db.add_color(hex, &new_name);
    new_name
}

fn generate_artistic_name(hex: &str, db: &ColorMaps) -> String {
    let (r, g, b) = parse_rgb(hex);
    let (hue, saturation, value) = rgb_to_hsv(r, g, b);

    let mut hasher = DefaultHasher::new();
    hex.hash(&mut hasher);
    let seed = hasher.finish();
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

    // Enhanced color characteristic analysis
    let is_bright = value > 0.85;
    let is_dark = value < 0.25;
    let is_vivid = saturation > 0.8;
    let is_muted = saturation < 0.3;
    let is_warm = (hue >= 0.0 && hue <= 60.0) || (hue >= 320.0 && hue <= 360.0);
    let is_cool = hue >= 180.0 && hue <= 300.0;
    let is_natural = (hue >= 60.0 && hue <= 150.0) || saturation < 0.4;

    let mut word_pools: Vec<(&[&str], f64)> = Vec::new();

    // Primary characteristics - highest weights
    if is_bright {
        word_pools.push((LUMINOUS_TERMS, 1.0));
        word_pools.push((ATMOSPHERIC_TERMS, 0.8));
    }
    if is_dark {
        word_pools.push((DEEP_TERMS, 1.0));
        word_pools.push((TEMPORAL_TERMS, 0.7));
    }

    // Color temperature and intensity
    if is_vivid {
        if is_warm {
            word_pools.push((WARM_TERMS, 0.9));
            if value > 0.6 {
                word_pools.push((LUMINOUS_TERMS, 0.7));
            } else {
                word_pools.push((EMOTIONAL_TERMS, 0.4));
            }
        } else if is_cool {
            word_pools.push((COOL_TERMS, 0.9));
            if value < 0.4 {
                word_pools.push((DEEP_TERMS, 0.7));
            }
        }
    }

    // Muted and natural tones
    if is_muted {
        word_pools.push((NEUTRAL_TERMS, 0.8));
        if is_natural {
            word_pools.push((BOTANICAL_TERMS, 0.7));
        }
    }

    // Hue-specific associations
    match hue as i32 {
        0..=60 => {
            word_pools.push((WARM_TERMS, 0.8));
            if saturation > 0.5 {
                word_pools.push((TEMPORAL_TERMS, 0.6));
                word_pools.push((ATMOSPHERIC_TERMS, 0.5));
            }
        }
        61..=180 => {
            word_pools.push((BOTANICAL_TERMS, 0.9));
            if value > 0.6 {
                word_pools.push((ATMOSPHERIC_TERMS, 0.7));
            } else {
                word_pools.push((DEEP_TERMS, 0.6));
            }
        }
        181..=300 => {
            word_pools.push((COOL_TERMS, 0.8));
            if value < 0.4 {
                word_pools.push((DEEP_TERMS, 0.7));
            } else {
                word_pools.push((ATMOSPHERIC_TERMS, 0.6));
            }
        }
        _ => {
            word_pools.push((WARM_TERMS, 0.7));
            if saturation < 0.3 {
                word_pools.push((NEUTRAL_TERMS, 0.8));
            } else {
                word_pools.push((TEMPORAL_TERMS, 0.6));
            }
        }
    }

    // Contextual adjustments
    if is_bright && is_vivid {
        word_pools.push((LUMINOUS_TERMS, 0.9));
    }
    if is_dark && is_cool {
        word_pools.push((DEEP_TERMS, 0.9));
    }
    if is_muted && value > 0.5 {
        word_pools.push((ATMOSPHERIC_TERMS, 0.7));
    }
    if is_natural && saturation < 0.5 {
        word_pools.push((BOTANICAL_TERMS, 0.8));
    }
    if is_natural && is_vivid {
        word_pools.push((NATURAL_ELEMENTS, 0.8));
    }

    // Initialize collections with proper types
    let mut selected_words: Vec<&str> = Vec::new();
    let mut used_categories: HashSet<*const [&str]> = HashSet::new();
    let mut attempts: i32 = 0;
    let max_attempts: i32 = 15;

    while selected_words.len() < 2 && attempts < max_attempts {
        word_pools.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        for (pool, weight) in &word_pools {
            let effective_weight = weight * (1.0 - (attempts as f64 / max_attempts as f64));
            if rng.gen::<f64>() < effective_weight {
                if let Some(word) = pool.choose(&mut rng) {
                    let category = *pool as *const [&str];
                    if !used_categories.contains(&category) && !selected_words.contains(word) {
                        selected_words.push(word);
                        used_categories.insert(category);
                        if selected_words.len() == 2 {
                            break;
                        }
                    }
                }
            }
        }
        attempts += 1;
    }

    // Fallback selection with proper types
    while selected_words.len() < 2 {
        for (pool, _) in word_pools.iter().take(3) {
            if let Some(word) = pool.choose(&mut rng) {
                if !selected_words.contains(word) {
                    selected_words.push(word);
                    if selected_words.len() == 2 {
                        break;
                    }
                }
            }
        }
    }

    // Fixed string comparison
    let mut name = format!("{}{}", selected_words[0], selected_words[1]);
    let mut uniqueness_attempts = 0;
    while db.name_exists(&name) && uniqueness_attempts < 10 {
        for (pool, _) in word_pools.iter().take(3) {
            if let Some(new_word) = pool.choose(&mut rng) {
                if *new_word != selected_words[0] {
                    name = format!("{}{}", selected_words[0], new_word);
                    if !db.name_exists(&name) {
                        return name;
                    }
                }
            }
        }
        uniqueness_attempts += 1;
    }

    name
}

fn load_color_db(path: &str) -> Result<ColorMaps, Box<dyn std::error::Error>> {
    if !Path::new(path).exists() {
        let empty = ColorMaps::default();
        save_color_db(path, &empty)?;
        return Ok(empty);
    }

    let data = fs::read_to_string(path)?;
    let db: ColorMaps = serde_json::from_str(&data)?;
    Ok(db)
}

fn save_color_db(path: &str, db: &ColorMaps) -> Result<(), Box<dyn std::error::Error>> {
    let json_str = serde_json::to_string_pretty(db)?;
    fs::write(path, json_str)?;
    Ok(())
}

fn extract_colors(content: &str) -> Vec<String> {
    let re = Regex::new(r"fill:\s*([#][0-9A-Fa-f]{6})").unwrap();
    let mut unique = HashSet::new();

    for cap in re.captures_iter(content) {
        unique.insert(cap[1].to_uppercase());
    }

    let mut colors: Vec<_> = unique.into_iter().collect();
    colors.retain(|c| c != "#FFFFFF");

    colors.sort_by(|a, b| {
        approximate_luminance(a)
            .partial_cmp(&approximate_luminance(b))
            .unwrap()
    });
    colors
}

fn approximate_luminance(hex: &str) -> f64 {
    let (r, g, b) = parse_rgb(hex);
    (0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64) / 255.0
}

/// Parses a hex color string (e.g., "#FFA500") into its RGB components.
/// Assumes the hex string is in the format "#RRGGBB".
fn parse_rgb(hex: &str) -> (u8, u8, u8) {
    let r = u8::from_str_radix(&hex[1..3], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[3..5], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[5..7], 16).unwrap_or(0);
    (r, g, b)
}

/// Converts RGB components to HSV.
/// Returns (Hue in degrees [0, 360), Saturation [0,1], Value [0,1]).
fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let rf = r as f64 / 255.0;
    let gf = g as f64 / 255.0;
    let bf = b as f64 / 255.0;

    let max = rf.max(gf).max(bf);
    let min = rf.min(gf).min(bf);
    let delta = max - min;

    // Calculate Hue
    let mut h = if delta < 1e-8 {
        0.0
    } else if (max - rf).abs() < 1e-8 {
        60.0 * (((gf - bf) / delta) % 6.0)
    } else if (max - gf).abs() < 1e-8 {
        60.0 * (((bf - rf) / delta) + 2.0)
    } else {
        60.0 * (((rf - gf) / delta) + 4.0)
    };
    if h < 0.0 {
        h += 360.0;
    }

    // Calculate Saturation
    let s = if max < 1e-8 { 0.0 } else { delta / max };

    // Value
    let v = max;

    (h, s, v)
}

/// Adjusts hue to wrap around for hues >= 345.0 degrees.
/// This ensures that hues near 360° appear next to hues near 0°, creating a seamless rainbow.
// fn adjust_hue(h: f64) -> f64 {
//     if h >= 345.0 {
//         h - 360.0
//     } else {
//         h
//     }
// }

/// Sanitizes a string to be a valid Mermaid class name.
/// Replaces non-alphanumeric characters with underscores and ensures it doesn't start with a digit.
fn sanitize_class_name(name: &str) -> String {
    let mut sanitized = String::new();
    for c in name.chars() {
        if c.is_alphanumeric() {
            sanitized.push(c);
        } else {
            sanitized.push('_');
        }
    }
    // If the first character is a digit, prepend an underscore
    if sanitized.chars().next().map_or(false, |c| c.is_digit(10)) {
        sanitized = format!("_{}", sanitized);
    }
    sanitized
}

/// Determines the appropriate text color (black or white) based on the luminance of the fill color.
/// Uses the YIQ formula to calculate luminance.
fn determine_text_color(r: u8, g: u8, b: u8) -> String {
    let yiq = (r as f64 * 299.0 + g as f64 * 587.0 + b as f64 * 114.0) / 1000.0;
    if yiq >= 128.0 {
        "#000000".to_string() // Black text for light backgrounds
    } else {
        "#FFFFFF".to_string() // White text for dark backgrounds
    }
}

/// Generates a Mermaid flowchart string from a list of colors and their names.
/// Ensures uniform node sizes, vertical alignment, and centralized class definitions.
/// Implements dynamic text color based on background luminance.
fn generate_mermaid(color_and_name: Vec<(String, String)>) -> String {
    // First find the longest label length
    let max_length = color_and_name
        .iter()
        .map(|(hex, name)| format!("{}:    {}", name, hex).len())
        .max()
        .unwrap_or(0);

    let mut out = String::new();
    out.push_str("%%{ init: { 'flowchart': { 'nodeSpacing': -2, 'rankSpacing': 0.6, 'htmlLabels': true } } }%%\n");
    out.push_str("flowchart TB\n\n");

    out.push_str(
        "    classDef default margin:0,padding:0px,stroke:none,display:flex,align-items:center,justify-content:center,white-space:pre\n\n",
    );

    for (hex, name) in color_and_name.iter() {
        let sanitized_class_name = sanitize_class_name(name);
        let (r, g, b) = parse_rgb(hex);
        let text_color = determine_text_color(r, g, b);
        out.push_str(&format!(
            "    classDef {} fill:{},color:{}\n",
            sanitized_class_name, hex, text_color
        ));
    }

    out.push_str("\n");

    for (i, (hex, name)) in color_and_name.iter().enumerate() {
        let node_id = format!("color{}", i);
        let base_text = format!("{}:    {}", name, hex);
        let padding_needed = max_length - base_text.len();
        let padded_text = format!("{}{}", base_text, " ".repeat(padding_needed));
        let sanitized_class_name = sanitize_class_name(name);

        if i > 0 {
            out.push_str(&format!("    color{} --- {}\n", i - 1, node_id));
        }

        out.push_str(&format!(
            "    {}[{}]:::{}\n",
            node_id, padded_text, sanitized_class_name
        ));
    }

    out.push_str("\n    linkStyle default stroke:none\n");

    out
}
