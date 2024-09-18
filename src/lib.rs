use std::fmt::Display;
use pyo3::prelude::*;
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug,Clone,EnumIter)]
pub enum Input {
    XPos,
    YPos,
    Zero,
    Const(f64),
}

impl Distribution<Input> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Input {
        let mut input = Input::iter().nth(rng.gen_range(0..Input::iter().len())).unwrap();
        if let Input::Const(..) = input {
            let f: f64 = rand::distributions::Standard::sample(&self, rng);
            input = Input::Const((f-0.5)*2.0);
        }
        input
    }
}

#[derive(Debug,EnumIter,Clone)]
pub enum UniVarFunc {
    Cos,
    Sqr,
    Abs,
    Neg,
}

impl Distribution<UniVarFunc> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> UniVarFunc {
        UniVarFunc::iter().nth(rng.gen_range(0..UniVarFunc::iter().len())).unwrap()
    }
}

impl UniVarFunc {
    fn func(&self) -> fn(f64) -> f64 {
        match self {
            UniVarFunc::Cos => f64::cos,
            UniVarFunc::Abs => f64::abs,
            UniVarFunc::Neg => |x| -x,
            UniVarFunc::Sqr => |x| x*x,
        }
    }
    fn display(&self) -> String {
        match self {
            UniVarFunc::Cos => "cos",
            UniVarFunc::Sqr => "sqr",
            UniVarFunc::Abs => "abs",
            UniVarFunc::Neg => "neg",
        }.to_string()
    }
}

#[derive(Debug,EnumIter,Clone)]
pub enum BiVarFunc {
    Sum,
    Diff,
    Prod,
    Max,
}

impl Distribution<BiVarFunc> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> BiVarFunc {
        BiVarFunc::iter().nth(rng.gen_range(0..BiVarFunc::iter().len())).unwrap()
    }
}

impl BiVarFunc {
    fn func(&self) -> fn(f64,f64) -> f64 {
        match self {
            BiVarFunc::Sum => |x,y| x+y,
            BiVarFunc::Diff => |x,y| x-y,
            BiVarFunc::Prod => |x,y| x*y,
            BiVarFunc::Max => f64::max,
        }
    }
    fn display(&self) -> String {
        match self {
            BiVarFunc::Sum => "sum",
            BiVarFunc::Diff => "dif",
            BiVarFunc::Prod => "prd",
            BiVarFunc::Max => "max",
        }.to_string()
    }
}

#[pyfunction]
fn random_tree(depth: usize) -> PyResult<Tree> {
    let tree: Tree = Tree {
        root: match (rand::random::<usize>() % 3, depth) {
            (0,_) | (_,0) => Node::Input(rand::random()),
            (1,_) => Node::UniVarFunc {
                child: Box::new(random_tree(depth-1)?.root),
                func: {
                    let uvf: UniVarFunc = rand::random();
                    uvf
                }
            },
            (2,_) => Node::BiVarFunc {
                child_a: Box::new(random_tree(depth-1)?.root),
                child_b: Box::new(random_tree(depth-1)?.root),
                func: {
                    let bvf: BiVarFunc = rand::random();
                    bvf
                },
            },
            _ => unreachable!(),
        }
    };
    Ok(tree)
}

#[pyfunction]
fn mutate(tree: &Tree) -> PyResult<Tree> {
    let mut new_tree = tree.clone();
    let mut new_tree_children = new_tree.root.get_all_children();
    let len = new_tree_children.len();
    let node_to_replace: &mut Node = new_tree_children[rand::random::<usize>() % len];
    let new_subtree = random_tree(tree.root.depth())?;
    *node_to_replace = new_subtree.root;
    Ok(new_tree)
}

#[pyfunction]
fn breed(tree_a: &Tree, tree_b: &Tree) -> PyResult<Tree> {
    let mut new_tree = tree_b.clone();
    let mut new_tree_children = new_tree.root.get_all_children();
    let mut donor_tree = tree_a.clone();
    let tree_a_children = donor_tree.root.get_all_children();
    let new_subtree = tree_a_children[
        rand::random::<usize>() % tree_a_children.len()
    ].clone();
    let len = new_tree_children.len();
    let node_to_replace: &mut Node = new_tree_children[rand::random::<usize>() % len];
    *node_to_replace = new_subtree;
    Ok(new_tree)
}

/// A Python module implemented in Rust.
#[pymodule]
fn geneprog(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(breed, m)?)?;
    m.add_class::<Tree>()?;
    m.add_function(wrap_pyfunction!(random_tree, m)?)?;
    m.add_function(wrap_pyfunction!(mutate, m)?)?;
    Ok(())
}

#[pyclass]
#[derive(Clone,Debug)]
pub struct Tree {
    root: Node,
}

#[pymethods]
impl Tree {
    pub fn eval(&self, x: f64, y: f64) -> f64 {
        self.root.eval(x, y)
    }
    pub fn show(&self) -> String {
        self.root.display()
    }
}

#[derive(Debug,Clone)]
pub enum Node {
    Input(Input),
    UniVarFunc{
        child: Box<Node>,
        func: UniVarFunc,
    },
    BiVarFunc{
        child_a: Box<Node>,
        child_b: Box<Node>,
        func: BiVarFunc,
    },
}


impl Node {
    pub fn eval(&self, x: f64, y: f64) -> f64 {
        match self {
            Node::Input(input) => match input {
                Input::XPos => x,
                Input::YPos => y,
                Input::Const(f) => *f,
                Input::Zero => 0.0,
            },
            Node::UniVarFunc { child, func } => func.func()(child.eval(x,y)),
            Node::BiVarFunc { child_a, child_b, func } => func.func()(
                child_a.eval(x,y), child_b.eval(x,y)
            )
        }
    }

    pub fn from_uvf(func: UniVarFunc, child: Node) -> Self {
        Node::UniVarFunc { child: Box::new(child), func }
    }

    pub fn from_bvf(
        func: BiVarFunc, child_a: Node, child_b: Node
    ) -> Self {
        Node::BiVarFunc {
            child_a: Box::new(child_a),
            child_b: Box::new(child_b),
            func,
        }
    }

    pub fn input(val: f64) -> Self {
        Node::Input(Input::Const(val))
    }

    pub fn get_children(&self) -> Vec<Node> {
        match self {
            Node::Input(_) => vec![],
            Node::UniVarFunc { child, .. } => vec![*child.to_owned()],
            Node::BiVarFunc { child_a, child_b, .. } => {
                vec![*child_a.to_owned(), *child_b.to_owned()]
            },
        }
    }

    pub fn get_all_children(&mut self) -> Vec<&mut Node> {
        let mut out: Vec<&mut Node> = vec![];
        match self {
            Node::Input(_) => out.push(self),
            Node::UniVarFunc { ref mut child, .. } => {
                out.append(&mut child.get_all_children());
            },
            Node::BiVarFunc { ref mut child_a, ref mut child_b, .. } => {
                out.append(&mut child_a.get_all_children());
                out.append(&mut child_b.get_all_children());
            },
        }
        out
    }

    pub fn set_children(&mut self, children: &[Node]) -> Result<(),String> {
        if children.len() != self.get_children().len() {
            return Err("children length incompatible".to_string());
        }
        match self {
            Node::Input(_) => {},
            Node::UniVarFunc { child, .. } => {
                *child = Box::new(children[0].clone());
            },
            Node::BiVarFunc { child_a, child_b, .. } => {
                *child_a = Box::new(children[0].clone());
                *child_b = Box::new(children[1].clone());
            }
        }
        Ok(())
    }

    pub fn depth(&self) -> usize {
        match self {
            Node::Input(_) => 0,
            Node::UniVarFunc { child, .. } => child.depth() + 1,
            Node::BiVarFunc { child_a, child_b, .. } => usize::max(
                child_a.depth(),
                child_b.depth(),
            ) + 1,
        }
    }

    pub fn new_rand(
        inputset: Vec<Input>,
        uvfuncset: Vec<UniVarFunc>,
        bvfuncset: Vec<BiVarFunc>,
        maxdepth: usize,
    ) -> Node {
        let mut root = Node::Input(
            inputset[rand::random::<usize>() % inputset.len()].clone()
        );
        while root.depth() < maxdepth {
            if rand::random::<u8>() % 2 == 0 {
                let func = uvfuncset[
                    rand::random::<usize>() % uvfuncset.len()
                ].clone();
                root = Node::from_uvf(func, root.clone());
            } else {
                let func = bvfuncset[
                    rand::random::<usize>() % bvfuncset.len()
                ].clone();
                root = Node::from_bvf(func, root.clone(), Node::Input(
                    inputset[rand::random::<usize>() % inputset.len()].clone()
                ));
            }
        }
        root
    }
    pub fn display(&self) -> String {
        match self {
            Node::Input(input) => match input {
                Input::XPos => "x".to_string(),
                Input::YPos => "y".to_string(),
                Input::Const(c) => format!("{:0.5}", c),
                Input::Zero => "0".to_string(),
            },
            Node::UniVarFunc { child, func } => {
                format!(
                    "{}({})",
                    func.display(),
                    child.display()
                )
            }
            Node::BiVarFunc { child_a, child_b, func } => {
                format!(
                    "{}({},{})",
                    func.display(),
                    child_a.display(),
                    child_b.display(),
                )
            },
        }
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::Input(_) => write!(f, "x")?,
            Node::UniVarFunc { child, .. } => {
                write!(f,"u(")?;
                child.fmt(f)?;
                write!(f,")")?;
            },
            Node::BiVarFunc { child_a, child_b, .. } => {
                write!(f,"b(")?;
                child_a.fmt(f)?;
                write!(f,",")?;
                child_b.fmt(f)?;
                write!(f,")")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_tree() {
        let input = Node::Input(Input::Const(10.0));
        assert_eq!(input.eval(0.0,0.0), 10.0);
    }

    #[test]
    fn univarfunc() {
        let input = Node::Input(Input::Const(-10.0));
        let func = Node::UniVarFunc {
            child: Box::new(input),
            func: UniVarFunc::Abs,
        };
        assert_eq!(func.eval(0.0,0.0), 10.0);
    }

    #[test]
    fn bivarfunc() {
        let input_a = Node::Input(Input::XPos);
        let input_b = Node::Input(Input::YPos);
        let func = Node::BiVarFunc {
            child_a: Box::new(input_a),
            child_b: Box::new(input_b),
            func: BiVarFunc::Sum,
        };
        assert_eq!(func.eval(3.0, 4.0), 7.0);
    }

    #[test]
    fn mixed() {
        let input_a = Node::Input(Input::XPos);
        let input_b = Node::Input(Input::YPos);

        let func1a = Node::UniVarFunc {
            child: Box::new(input_a),
            func: UniVarFunc::Abs,
        };
        let func1b = Node::UniVarFunc {
            child: Box::new(input_b),
            func: UniVarFunc::Abs,
        };
        
        let func2 = Node::BiVarFunc {
            child_a: Box::new(func1a),
            child_b: Box::new(func1b),
            func: BiVarFunc::Sum,
        };

        assert_eq!(func2.eval(-3.0,-4.0), 7.0);
    }

    #[test]
    fn mixed_fromsyntax() {
        let input_a = Node::Input(Input::XPos);
        let input_b = Node::Input(Input::YPos);

        let func1a = Node::from_uvf(UniVarFunc::Abs, input_a);
        let func1b = Node::from_uvf(UniVarFunc::Abs, input_b);
        
        let func2 = Node::from_bvf(BiVarFunc::Sum, func1a, func1b);
        assert_eq!(func2.eval(-3.0, -4.0), 7.0);
    }

    #[test]
    fn replace() {
        let input = Node::Input(Input::Const(3.0));
        let mut root = Node::from_uvf(UniVarFunc::Abs, input);
        assert_eq!(root.eval(0.0, 0.0), 3.0);
        root.set_children(&[Node::Input(Input::Const(-4.0))]).unwrap();
        assert_eq!(root.eval(0.0, 0.0), 4.0);
    }

    #[test]
    fn depth() {
        let input = Node::input(-3.0);
        let mut root = Node::from_uvf(UniVarFunc::Abs, input);
        assert_eq!(root.depth(), 1);
        root.set_children(&[root.clone()]).unwrap();
        assert_eq!(root.depth(), 2);
        root.set_children(&[root.clone()]).unwrap();
        assert_eq!(root.depth(), 3);
        root.set_children(&[root.clone()]).unwrap();
        assert_eq!(root.depth(), 4);
    }

    #[test]
    fn randnew() {
        let mut inputset: Vec<Input> = vec![];
        inputset.push(Input::Const(1.0));
        inputset.push(Input::Const(2.0));
        let mut uvfuncset: Vec<UniVarFunc> = vec![];
        uvfuncset.push(UniVarFunc::Abs);
        uvfuncset.push(UniVarFunc::Neg);
        uvfuncset.push(UniVarFunc::Cos);
        let mut bvfuncset: Vec<BiVarFunc> = vec![];
        bvfuncset.push(BiVarFunc::Sum);
        bvfuncset.push(BiVarFunc::Diff);
        bvfuncset.push(BiVarFunc::Prod);
        let node = Node::new_rand(inputset, uvfuncset, bvfuncset, 5);
        println!("{}", node);
        println!("{:?}", node);
    }
}