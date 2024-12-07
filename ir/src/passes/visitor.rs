use crate::ir2::{IsParent, Link, NodeType};

pub enum VisitOrder {
    Manual,
    DepthFirst,
    PostOrder,
}
pub trait VisitDefault {}
pub trait VisitContext {
    type Graph;
    fn visit(&mut self, graph: &mut Self::Graph, link: Link<NodeType>);
    fn as_stack_mut(&mut self) -> &mut Vec<Link<NodeType>>;
    fn boundary_roots(&self, graph: &Self::Graph) -> Link<Vec<Link<NodeType>>>;
    fn integrity_roots(&self, graph: &Self::Graph) -> Link<Vec<Link<NodeType>>>;
    fn visit_order(&self) -> VisitOrder;
}
pub trait Visit: VisitContext {
    fn run(&mut self, graph: &mut Self::Graph) {
        match self.visit_order() {
            VisitOrder::Manual => self.visit_manual(graph),
            VisitOrder::PostOrder => self.visit_postorder(graph),
            VisitOrder::DepthFirst => self.visit_depthfirst(graph),
        }
        while let Some(node_index) = self.next_node() {
            self.visit(graph, node_index);
        }
    }
    fn visit_manual(&mut self, graph: &mut Self::Graph) {
        for root_index in self
            .boundary_roots(graph)
            .borrow()
            .iter()
            .chain(self.integrity_roots(graph).borrow().iter())
        {
            self.visit(graph, root_index.clone());
        }
    }
    fn visit_postorder(&mut self, graph: &mut Self::Graph) {
        for root_index in self
            .boundary_roots(graph)
            .borrow()
            .iter()
            .chain(self.integrity_roots(graph).borrow().iter())
        {
            self.visit_later(root_index.clone());
            let mut last: Option<Link<NodeType>> = None;
            while let Some(link) = self.peek() {
                let children = link.get_children();
                if children.borrow().is_empty()
                    || last.is_some() && children.borrow().contains(&last.clone().unwrap())
                {
                    self.visit(graph, link.clone());
                    self.next_node();
                    last = Some(link.clone());
                } else {
                    for child in children.borrow().iter().rev() {
                        self.visit_later(child.clone());
                    }
                }
            }
        }
    }
    fn visit_depthfirst(&mut self, graph: &mut Self::Graph) {
        for root_index in self
            .boundary_roots(graph)
            .borrow()
            .iter()
            .chain(self.integrity_roots(graph).borrow().iter())
        {
            self.visit_later(root_index.clone());
            while let Some(link) = self.next_node() {
                let children = link.get_children();
                for child in children.borrow().iter().rev() {
                    self.visit_later(child.clone());
                }
                self.visit(graph, link);
            }
        }
    }
    fn peek(&mut self) -> Option<Link<NodeType>> {
        self.as_stack_mut().last().cloned()
    }
    fn next_node(&mut self) -> Option<Link<NodeType>> {
        self.as_stack_mut().pop()
    }
    fn visit_later(&mut self, link: Link<NodeType>) {
        self.as_stack_mut().push(link);
    }
}

impl<T> Visit for T where T: VisitContext + VisitDefault {}
