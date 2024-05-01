extern crate bam;
extern crate rust_htslib;
use indexmap::IndexMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use strsim::hamming;

// this is the struct that contains functions used to group umis per the directional method
pub struct Processor<'b> {
    pub umis: &'b Vec<String>,
}

impl<'b> Processor<'b> {
    // for getting all unique umi keys from adjacency list
    // visits a UMI, gets all UMIs grouped with it, then moves onto next UMI.
    // doesn't add duplicate UMIs
    pub fn depth_first_search(
        mut node: &'b String,
        adj_list: &IndexMap<&'b String, HashSet<&'b String>>,
    ) -> VecDeque<&'b String> {
        let mut searched: VecDeque<&String> = VecDeque::new();
        let mut queue: VecDeque<&String> = VecDeque::new();

        queue.push_back(node);
        searched.push_back(node);

        while queue.len() > 0 {
            node = &queue.pop_front().unwrap();
            if adj_list.contains_key(node) {
                for next_node in &adj_list[node] {
                    if !searched.contains(&next_node) {
                        queue.push_back(next_node);
                        searched.push_back(next_node);
                    }
                }
            }
        }
        return searched;
    }

    // gets edit distance (hamming distance) between two umis
    pub fn edit_distance(ua: &String, ub: &String) -> usize {
        return hamming(ua, ub).unwrap();
    }

    // groups umis via directional algorithm
    pub fn get_adj_list_directional(
        &self,
        counts: HashMap<&String, i32>,
        threshold: usize,
    ) -> IndexMap<&'b String, HashSet<&'b String>> {
        let mut adj_list: IndexMap<&'b String, HashSet<&'b String>> =
            IndexMap::with_capacity(self.umis.len());
        let mut i = 0;
        while i < self.umis.len() {
            let top = &self.umis[i];
            adj_list.entry(top).or_insert(HashSet::new());
            i += 1;
            let remainder = self.umis.iter();
            for sub in remainder {
                adj_list.entry(sub).or_insert(HashSet::new());
                println!{"                  {} count {}        {} count {}", top, counts.get(top).unwrap(), sub, counts.get(sub).unwrap()};
                println!{"edit distance {} ", Processor::edit_distance(top, sub)}
                if Processor::edit_distance(top, sub) <= threshold && top != sub {
                    println!{"hamming checked!"}
                    if *counts.get(top).unwrap() >= (counts.get(sub).unwrap() * 2 - 1) {
                        adj_list[top].insert(sub);
                        println!{"top key generated"};
                    } else if *counts.get(sub).unwrap() >= (counts.get(top).unwrap() * 2 - 1) {
                        adj_list[sub].insert(top);
                        println!{"sub key generated"};
                    }
                } else {
                }
            }
        }
        return adj_list;
    }

    // return a list of lists, comprising a UMI
    // with a list of grouped UMIs.
    // via breadth-first-search
    // this is fed directly into the main_grouper function
    pub fn get_connected_components(
        &self,
        adj_list: IndexMap<&'b String, HashSet<&'b String>>,
    ) -> Option<Vec<VecDeque<&String>>> {
        let mut components: Vec<VecDeque<&String>> = Vec::new();
        let mut found: Vec<&String> = Vec::new();

        if adj_list.len() > 0 {
            for node in adj_list.keys() {
                if !found.contains(node) {
                    let component = Processor::depth_first_search(node, &adj_list);
                    found.extend(&component);
                    components.push(component);
                }
            }
            return Some(components);
        } else {
            return None;
        }
    }

    // get a list of UMIs, each with their own list of UMIs belonging to their group
    pub fn group_directional(&self, clusters: Vec<VecDeque<&'b String>>) -> Vec<Vec<&'b String>> {
        // println! {"generating groups...."};
        let mut observed: Vec<&String> = Vec::new();
        let mut groups: Vec<Vec<&String>> = Vec::new();

        for cluster in clusters {
            if cluster.len() == 1 {
                observed.push(&cluster.get(0).unwrap());
                groups.push(cluster.into());
            } else {
                let mut temp_cluster: Vec<&String> = Vec::new();

                for node in cluster {
                    if !observed.contains(&&node) {
                        temp_cluster.push(node);
                        observed.push(node);
                    }
                }
                groups.push(temp_cluster);
            }
        }
        return groups;
    }

    // driver code for directional method,
    // and UMI organization and grouping
    // pub fn main_grouper(&self, counts: HashMap<String, i32>) -> Option<Vec<Vec<String>>> {
    // pub fn main_grouper(&self, counts: HashMap<String, i32>) -> (Option<Vec<Vec<String>>>, Option<Vec<&String>>) {
    pub fn main_grouper(&self, counts: HashMap<&String, i32>) -> Option<Vec<Vec<&String>>> {
        let directional_output = self.get_adj_list_directional(counts, 1);
        let adj_list = directional_output;
        // println!{"adj list {:?}", adj_list};
        let final_umis;
        if adj_list.len() > 0 {
            let clusters = self.get_connected_components(adj_list).unwrap();
            final_umis = Some(self.group_directional(clusters));
        } else {
            final_umis = None;
        }

        // return (final_umis, Some(singletons));
        return final_umis;
    }
}
