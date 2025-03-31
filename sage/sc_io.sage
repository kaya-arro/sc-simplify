from sys import stdin
from sage.topology.simplicial_complex import SimplicialComplex

def read_sc_pair(filename):
    with open(f"{filename}", "r") as f:
        rfs = []
        bfs = []
        n = 0
        fs = [rfs, bfs]
        for line in f.readlines():
            if not line:
                n += 1
            facet = list(map(int, line.split()))
            fs[n].append(facet)
    return tuple(map(SimplicialComplex, fs))

def read_sc_pair_from_stdin():
    rfs = []
    bfs = []
    n = 0
    fs = [rfs, bfs]
    for line in stdin.readlines():
        if not line:
            n += 1
        facet = list(map(int, line.split()))
        fs[n].append(facet)
    return tuple(map(SimplicialComplex, fs))

def write_sc(sc, filename=None):
    if filename is None:
        filename = sc.__repr__() + ".sc"
    with open(f"{filename}", "w") as f:
        for facet in sc._facets:
            str = ""
            for v in facet.tuple():
                str += f" {v}"
            f.write(str[1:] + "\n")
