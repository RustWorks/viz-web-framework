use viz_core::{BoxHandler, Method};

use crate::{PathTree, Route, Router};

#[derive(Default)]
pub struct Tree(Vec<(Method, PathTree<BoxHandler>)>);

impl Tree {
    pub fn find<'a>(
        &'a self,
        method: &'a Method,
        path: &'a str,
    ) -> Option<(&'a BoxHandler, Vec<(&'a str, &'a str)>)> {
        self.0
            .iter()
            .find_map(|(m, t)| if m == method { Some(t) } else { None })
            .and_then(|t| t.find(path))
    }

    pub fn into_inner(self) -> Vec<(Method, PathTree<BoxHandler>)> {
        self.0
    }
}

impl AsRef<Vec<(Method, PathTree<BoxHandler>)>> for Tree {
    fn as_ref(&self) -> &Vec<(Method, PathTree<BoxHandler>)> {
        &self.0
    }
}

impl AsMut<Vec<(Method, PathTree<BoxHandler>)>> for Tree {
    fn as_mut(&mut self) -> &mut Vec<(Method, PathTree<BoxHandler>)> {
        &mut self.0
    }
}

impl From<Router> for Tree {
    fn from(router: Router) -> Self {
        let mut tree = Tree::default();
        if let Some(routes) = router.routes {
            for (mut path, Route { methods }) in routes {
                if !path.starts_with('/') {
                    path.insert(0, '/');
                }
                for (method, handler) in methods {
                    match tree.as_mut().iter_mut().find_map(|(m, t)| {
                        if *m == method {
                            Some(t)
                        } else {
                            None
                        }
                    }) {
                        Some(t) => {
                            t.insert(&path, handler);
                        }
                        None => {
                            let mut t = PathTree::new();
                            t.insert(&path, handler);
                            tree.as_mut().push((method, t));
                        }
                    }
                }
            }
        }
        tree
    }
}
