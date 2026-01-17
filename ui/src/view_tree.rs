use iced::{Element, Length};
use iced::widget::{column, row, text, container};
use crate::editor::{view_node::Node, ViewNode, Parent, Leaf};

pub fn render_view_tree<'a, Message>(node: &'a ViewNode) -> Element<'a, Message> 
where 
    Message: 'a + Clone 
{
    match &node.node {
        Some(Node::Leaf(leaf)) => render_leaf(leaf),
        Some(Node::Parent(parent)) => render_parent(parent),
        None => container(text("Empty Layout")).width(Length::Fill).height(Length::Fill).into(),
    }
}

fn render_leaf<'a, Message>(leaf: &'a Leaf) -> Element<'a, Message> 
where 
    Message: 'a + Clone
{
    container(
        text(format!("View ID: {}\nBuffer: {:?}", leaf.id, leaf.buffer_id)) // Corrected newline escape
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(10)
    // .style(...) // スタイルはThemeに依存するため一旦省略
    .into()
}

fn render_parent<'a, Message>(parent: &'a Parent) -> Element<'a, Message> 
where 
    Message: 'a + Clone 
{
    // TODO: parent.sizes を使って Length::FillPortion 等を適用する
    // 現状は単純な Row/Column で均等分割
    
    let children = parent.children.iter().map(|c| {
        render_view_tree(c)
    });

    if parent.direction == 0 { // HORIZONTAL
        row(children).spacing(2).width(Length::Fill).height(Length::Fill).into()
    } else {
        column(children).spacing(2).width(Length::Fill).height(Length::Fill).into()
    }
}