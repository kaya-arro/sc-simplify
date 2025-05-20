struct BiComplex {
    cells: Vec<SCHashSet<Point>>;
    faces: SCHashMap<Point, SCHashMap<Point, bool>>;
    cofaces: SCHashMap<Point, SCHashSet<Point>>;
}

trait ChainComplex;

struct ModChainComplex;

impl ChainComplex for ModChainComplex;

impl From<BiComplex> for

struct ModTwoChainComplex;

impl ChainComplex for ModTwoChainComplex;

impl From<BiComplex> for ModTwoChainComplex;
