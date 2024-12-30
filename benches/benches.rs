use std::str::FromStr;

use criterion::{self, criterion_group, criterion_main, Criterion};
use tree_xml::node::Node;

const DEFINITION_XML_STR: &str = include_str!("../resources/definition.xml");
const JOIN_XML_STR: &str = include_str!("../resources/join.xml");
const SCORES_XML_STR: &str = include_str!("../resources/scores.xml");
const WINNER_XML_STR: &str = include_str!("../resources/winner.xml");
const LARGE_XML_STR: &str = include_str!("../resources/large.xml");

fn parse_definition_str_to_node(c: &mut Criterion) {
    c.bench_function("definition", |b| {
        b.iter(|| Node::from_str(DEFINITION_XML_STR))
    });
}

fn parse_join_str_to_node(c: &mut Criterion) {
    c.bench_function("join", |b| b.iter(|| Node::from_str(JOIN_XML_STR)));
}

fn parse_scores_str_to_node(c: &mut Criterion) {
    c.bench_function("scores", |b| b.iter(|| Node::from_str(SCORES_XML_STR)));
}

fn parse_winner_str_to_node(c: &mut Criterion) {
    c.bench_function("winner", |b| b.iter(|| Node::from_str(WINNER_XML_STR)));
}

fn parse_large_str_to_node(c: &mut Criterion) {
    c.bench_function("large", |b| b.iter(|| Node::from_str(LARGE_XML_STR)));
}

criterion_group!(
    benches,
    parse_definition_str_to_node,
    parse_join_str_to_node,
    parse_scores_str_to_node,
    parse_winner_str_to_node,
    parse_large_str_to_node
);
criterion_main!(benches);
