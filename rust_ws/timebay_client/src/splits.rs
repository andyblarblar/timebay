//! Lap timing logic and widgets

use crate::splits::SectorState::Incomplete;
use derive_more::{IsVariant, Unwrap};
use std::collections::BTreeSet;
use std::time::SystemTime;
use timebay_common::messages::DetectionMessage;

/// Splits widget
pub struct Splits {
    /// Nodes connected at the time of the start of this run. Must be len > 0.
    nodes: BTreeSet<u16>,
    /// Deltas between sensors. Node ids in these sectors are in ascending order
    sectors: Vec<Sector>,
    /// State of widget
    state: SplitState,
    /// Current sector we are evaluating
    current_sector: u16,
}

impl Splits {
    /// Creates a splits widget.
    ///
    /// Nodes is the current nodes connected at the time of the start of this split. This must have
    /// len > 0. If not, None is returned.
    pub fn new(nodes: BTreeSet<u16>) -> Option<Self> {
        if nodes.is_empty() {
            None
        } else {
            let sectors = {
                let mut last = None;
                let mut sec = vec![];

                // Sectors start at the first node, end at the next node, and then the next sector begins where the last ended
                for node in &nodes {
                    if last.is_none() {
                        last = Some(node);
                        continue;
                    };

                    sec.push(Sector::new(*last.unwrap(), *node));

                    last = Some(node);
                }

                // Add the last sector that wraps from the last node to the first. This covers the case of only one node, where this sector will be (0,0)
                sec.push(Sector::new(*last.unwrap(), *nodes.first().unwrap()));
                sec
            };

            Some(Self {
                nodes,
                sectors,
                state: SplitState::NotStarted,
                current_sector: 0,
            })
        }
    }

    /// Handles a node triggering. Returns the resulting state.
    pub fn handle_node_trigger(&mut self, msg: DetectionMessage) -> SplitState {
        // Widget can exist while completed
        if self.state.is_completed() {
            return self.state.clone();
        }

        // Ignore node triggers not connected at start
        if !self.nodes.contains(&msg.node_id) {
            log::trace!("Ignoring node trigger that was not connected at lap start");
            return self.state.clone();
        }

        // Start lap if first node triggers
        if self.state.is_not_started() {
            if self.sectors[0].nodes.0 == msg.node_id {
                log::trace!("Starting lap");
                self.state = SplitState::Running(msg.get_stamp());
            }
            return self.state.clone();
        }

        // If next expected node triggered, sector is complete
        if self.sectors[self.current_sector as usize].nodes.1 == msg.node_id {
            log::trace!("Completed sector {}", self.current_sector);

            // Use msg stamp to avoid including latency
            self.sectors[self.current_sector as usize].state =
                SectorState::Complete(msg.get_stamp());
        }
        // If we skipped over a node, we need to invalidate passed sectors (handling edge case of last sector)
        else if self.sectors[self.current_sector as usize].nodes.1 < msg.node_id
            && self.current_sector as usize != self.sectors.len() - 1
        {
            log::trace!("Skipped a node!");

            let sector_containing = self.get_sector_containing(msg.node_id).unwrap() as usize;

            log::trace!(
                "Invalidating sectors {}-{}",
                self.current_sector,
                sector_containing
            );

            // Invalidate passed sectors
            self.sectors[self.current_sector as usize..=sector_containing]
                .iter_mut()
                .for_each(|s| s.state = SectorState::Invalidated);
        }
        // If a passed node triggered again, just ignore
        else if self.sectors[self.current_sector as usize].nodes.1 > msg.node_id {
            log::trace!("Passed node triggered");
            return self.state.clone();
        }

        // Find next valid sector, if any
        let next_sect = self.get_next_sector();

        // Lap complete
        if next_sect.is_none() {
            log::trace!("Lap complete");

            self.state =
                SplitState::Completed(self.state.clone().unwrap_running(), msg.get_stamp());
        } else {
            self.current_sector = next_sect.unwrap();
            log::trace!("Advancing to sector {}", self.current_sector);
        }

        self.state.clone()
    }

    /// Gets the next incomplete sector, None if lap is complete
    fn get_next_sector(&self) -> Option<u16> {
        let mut curr = (self.current_sector + 1) as usize;

        loop {
            // No incomplete sectors left
            if curr > self.sectors.len() - 1 {
                return None;
            }

            if self.sectors[curr].state.is_incomplete() {
                return Some(curr as u16);
            }

            curr += 1;
        }
    }

    /// Gets the sector that contains the passed node as an end node
    fn get_sector_containing(&self, node: u16) -> Option<u16> {
        self.sectors
            .iter()
            .enumerate()
            .find(|(_, s)| s.nodes.1 == node)
            .map(|s| s.0 as u16)
    }

    pub fn get_state(&self) -> &SplitState {
        &self.state
    }

    //TODO add view, which is a colomn where each element is given by iterating over the sectors views, then has the time at the bottom
}

#[derive(IsVariant, Debug, Unwrap, Clone)]
pub enum SplitState {
    /// Waiting for the first sensor
    NotStarted,
    /// Lap is occurring, started at time
    Running(SystemTime),
    /// Lap is done, started at left time, ended at right time
    Completed(SystemTime, SystemTime),
}

/// Single timing unit in a split, bounded by 2 sensors
#[derive(Debug, Eq, PartialEq)]
struct Sector {
    state: SectorState,
    /// Nodes this sector is between
    nodes: (u16, u16),
}

impl Sector {
    pub fn new(starting_node: u16, ending_node: u16) -> Self {
        Self {
            state: Incomplete,
            nodes: (starting_node, ending_node),
        }
    }

    // TODO add view
}

#[derive(Debug, Eq, PartialEq, IsVariant)]
enum SectorState {
    /// Sector was skipped
    Invalidated,
    /// Node in sector disconnected
    Disconnected,
    /// Node has not been completed yet
    Incomplete,
    /// Sector was completed at the contained absolute time
    Complete(SystemTime),
}

#[cfg(test)]
mod tests {
    use crate::splits::{Sector, Splits};
    use std::collections::BTreeSet;
    use timebay_common::messages::DetectionMessage;

    #[test]
    fn sectors_are_correct() {
        let splits = Splits::new(BTreeSet::from_iter(1u16..=3)).unwrap();

        assert_eq!(
            splits.sectors,
            vec![Sector::new(1, 2), Sector::new(2, 3), Sector::new(3, 1)]
        );

        // Out of order
        let splits = Splits::new(BTreeSet::from_iter([4, 3, 7, 2].into_iter())).unwrap();

        assert_eq!(
            splits.sectors,
            vec![
                Sector::new(2, 3),
                Sector::new(3, 4),
                Sector::new(4, 7),
                Sector::new(7, 2)
            ]
        );

        // Single node
        let splits = Splits::new(BTreeSet::from_iter([1].into_iter())).unwrap();

        assert_eq!(splits.sectors, vec![Sector::new(1, 1),]);
    }

    #[test]
    fn trigger_handling_works() {
        let mut splits = Splits::new(BTreeSet::from_iter(1u16..=3)).unwrap();

        assert_eq!(splits.get_sector_containing(3).unwrap(), 1);

        // Only first node starts run
        assert!(splits
            .handle_node_trigger(DetectionMessage::new(0, 10, 1, 0))
            .is_not_started());
        assert!(splits
            .handle_node_trigger(DetectionMessage::new(2, 10, 1, 0))
            .is_not_started());
        assert!(splits
            .handle_node_trigger(DetectionMessage::new(1, 10, 1, 0))
            .is_running());

        // Trigger the 3rd node, invalidating 1-2 and 2-3, leaving us in the last sector of 3-1
        splits.handle_node_trigger(DetectionMessage::new(3, 10, 2, 0));
        assert!(splits.sectors[0].state.is_invalidated());
        assert!(splits.sectors[1].state.is_invalidated());
        assert!(splits.sectors[2].state.is_incomplete());
        assert_eq!(splits.current_sector, 2);

        // Trigger old node to ensure last node is handled same as rest
        splits.handle_node_trigger(DetectionMessage::new(3, 10, 2, 0));
        assert_eq!(splits.current_sector, 2);
        splits.handle_node_trigger(DetectionMessage::new(2, 10, 2, 0));
        assert_eq!(splits.current_sector, 2);

        // Finish run
        assert!(splits
            .handle_node_trigger(DetectionMessage::new(1, 11, 2, 0))
            .is_completed());
    }
}
