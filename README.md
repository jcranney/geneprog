# geneprog
genetric programming in rust

## Install
While this is in the v0.0.1 stage, I'm sticking to github only. If it proves to
be useful, I'll polish it up and make it cargo installable. For now, installation
is:
```bash
git clone https://github.com/jcranney/geneprog.git
cd geneprog
maturin develop --release
```

For the above to work, you will need [pyo3/maturin](https://github.com/pyo3/maturin)
installed, e.g.:

```bash
pip install maturin
```

## Usage
For an example of how to use this package, see [example.py](https://github.com/jcranney/geneprog/blob/main/example.py). To check that everything install properly, you can simply do:
```python
>>> import geneprog
>>> tree = geneprog.random_tree(2)  # spawn a tree with max depth = 2
>>> print(tree.show())  # print the tree in its serial representation

neg(sqr(x))
```