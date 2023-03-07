//! Lap timing logic and widgets

use crate::splits::SectorState::Incomplete;
use cursive::align::HAlign;
use cursive::theme::Color;
use cursive::views::{DummyView, LinearLayout, Panel, TextView};
use derive_more::{IsVariant, Unwrap};
use itertools::izip;
use std::collections::BTreeSet;
use std::time::{Duration, SystemTime};
use timebay_common::messages::DetectionMessage;

/// Lap timing system implementation. This also serves as the splits widget via it's [`Splits::view`] function.
#[derive(Clone)]
pub struct Splits {
    /// Nodes connected at the time of the start of this run
    nodes: BTreeSet<u16>,
    /// Deltas between sensors. Node ids in these sectors are in ascending order, with the exception of the last sector.
    sectors: Vec<Sector>,
    /// State of widget
    state: SplitState,
    /// Current sector we are evaluating
    current_sector: usize,
}

impl Splits {
    /// Creates a split.
    ///
    /// Nodes is the current nodes connected at the time of the start of this split, which may be empty.
    /// New nodes can be connected while this widget is still in state [`SplitState::NotStarted`]. As
    /// soon as the first node is triggered, the connected nodes will be locked in.
    pub fn new(nodes: BTreeSet<u16>) -> Self {
        let sectors = Self::generate_sectors(&nodes);

        Self {
            nodes,
            sectors,
            state: SplitState::NotStarted,
            current_sector: 0,
        }
    }

    /// Creates sectors based off of the current node count.
    ///
    /// If no nodes are connected, this will be an empty vector.
    fn generate_sectors(nodes: &BTreeSet<u16>) -> Vec<Sector> {
        let mut last = None;
        let mut sec = vec![];

        // Sectors start at the first node, end at the next node, and then the next sector begins where the last ended
        for node in nodes {
            if last.is_none() {
                last = Some(node);
                continue;
            };

            sec.push(Sector::new(*last.unwrap(), *node));

            last = Some(node);
        }

        if !nodes.is_empty() {
            // Add the last sector that wraps from the last node to the first. This covers the case of only one node, where this sector will be (0,0)
            sec.push(Sector::new(*last.unwrap(), *nodes.first().unwrap()));
        }
        sec
    }

    /// Registers a new node with the system and recreates the sectors accordingly.
    ///
    /// This function will only run if the system is in state [`SplitState::NotStarted`]. Otherwise,
    /// the node will not be added and the function will return false.
    pub fn connect_node(&mut self, node: u16) -> bool {
        if self.state.is_not_started() {
            self.nodes.insert(node);
            self.sectors = Self::generate_sectors(&self.nodes);
            true
        } else {
            false
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
            if let Some(sector) = self.sectors.get(0) {
                if sector.nodes.0 == msg.node_id {
                    log::trace!("Starting lap");
                    self.state = SplitState::Running(msg.get_stamp());
                }
            }

            return self.state.clone();
        }

        // If next expected node triggered, sector is complete
        if self.get_current_sector().nodes.1 == msg.node_id {
            log::trace!("Completed sector {}", self.current_sector);

            // Use msg stamp to avoid including latency
            self.get_current_sector().state = SectorState::Complete(msg.get_stamp());
        }
        // If we skipped over a node, we need to invalidate passed sectors (handling edge case of last sector, where node id is descending)
        else if self.get_current_sector().nodes.1 < msg.node_id
            && self.current_sector != self.sectors.len() - 1
        {
            log::trace!("Skipped a node!");

            let sector_containing = self.get_sector_containing(msg.node_id).unwrap();

            log::trace!(
                "Invalidating sectors {}-{}",
                self.current_sector,
                sector_containing
            );

            // Invalidate passed sectors
            self.sectors[self.current_sector..=sector_containing]
                .iter_mut()
                .for_each(|s| s.state = SectorState::Invalidated);
        }
        // If a passed node triggered again, just ignore (accept edge case if it was rejected above, since node ordering is reversed there)
        else if self.get_current_sector().nodes.1 > msg.node_id
            || (self.get_current_sector().nodes.1 < msg.node_id
                && self.current_sector == self.sectors.len() - 1)
        {
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

    fn get_current_sector(&mut self) -> &mut Sector {
        &mut self.sectors[self.current_sector]
    }

    /// Gets the next incomplete sector idx, None if lap is complete
    fn get_next_sector(&self) -> Option<usize> {
        let mut curr = self.current_sector + 1;

        loop {
            // No incomplete sectors left
            if curr > self.sectors.len() - 1 {
                return None;
            }

            if self.sectors[curr].state.is_incomplete() {
                return Some(curr);
            }

            curr += 1;
        }
    }

    /// Gets the sector that contains the passed node as an end node, if such a sector exists
    fn get_sector_containing(&self, node: u16) -> Option<usize> {
        self.sectors
            .iter()
            .enumerate()
            .find(|(_, s)| s.nodes.1 == node)
            .map(|s| s.0)
    }

    /// Returns the current state of this lap
    pub fn get_state(&self) -> &SplitState {
        &self.state
    }

    /// Creates the view for this set of splits.
    ///
    /// The last lap can be passed to generate time diffs.
    pub fn view(&self, last_lap: &Option<Self>) -> impl cursive::view::View {
        let mut sectors = vec![];
        let mut times = vec![];
        let mut diffs = vec![];

        // These are parallel arrays to sectors, containing additional aggregated timing data
        let splits = self.get_sector_times();
        let diffs_t = last_lap.as_ref().map(|l| self.get_diffs(l));

        for (i, sector) in self.sectors.iter().enumerate() {
            // Add sector name
            let name = format!("Sector {}-{}", sector.nodes.0, sector.nodes.1);
            sectors.push(Panel::new(TextView::new(name)));

            // Add sector time if done
            match sector.state {
                SectorState::Invalidated => {
                    times.push(Panel::new(
                        TextView::new("INVALIDATED").style(Color::Rgb(255, 0, 0)),
                    ));
                    diffs.push(Panel::new(TextView::new("N/A")));
                }
                Incomplete => {
                    times.push(Panel::new(TextView::new("...")));
                    diffs.push(Panel::new(TextView::new("N/A")));
                }
                SectorState::Complete(_) => {
                    let sect_t = splits[i].unwrap();

                    times.push(Panel::new(TextView::new(Self::format_time(&sect_t))));

                    // Add diff time if diff is possible
                    if let Some(ref last_lap) = diffs_t {
                        if let Some(Some(diff)) = last_lap.get(i) {
                            diffs.push(Panel::new(TextView::new(Self::format_diff(diff)).style(
                                if *diff < 0 {
                                    Color::Rgb(10, 250, 10)
                                } else if *diff > 0 {
                                    Color::Rgb(255, 0, 0)
                                } else {
                                    Color::Rgb(0, 0, 0)
                                },
                            )));
                        }
                    } else {
                        diffs.push(Panel::new(TextView::new("N/A")));
                    }
                }
            }
        }

        // Make sure the table is never empty
        if self.sectors.is_empty() {
            sectors.push(Panel::new(TextView::new("No nodes connected...")));
            times.push(Panel::new(TextView::new("...")));
            diffs.push(Panel::new(TextView::new("N/A")));
        }

        let total_time = {
            let other = last_lap.as_ref().map(|l| l.get_total_time());
            let us = self.get_total_time();

            if let Some(us) = us {
                if let Some(Some(other)) = other {
                    let diff = &(us.as_millis() as i32 - other.as_millis() as i32);
                    LinearLayout::horizontal()
                        .child(TextView::new(Self::format_time(&us) + " "))
                        .child(TextView::new(Self::format_diff(diff)).style(if *diff < 0 {
                            Color::Rgb(10, 250, 10)
                        } else if *diff > 0 {
                            Color::Rgb(255, 0, 0)
                        } else {
                            Color::Rgb(0, 0, 0)
                        }))
                } else {
                    LinearLayout::horizontal().child(TextView::new(Self::format_time(&us)))
                }
            } else {
                LinearLayout::horizontal().child(TextView::new(String::from("0:00.0")))
            }
        };

        // Create our table out of horizontal views in a vertical view, forming a grid
        let sector_times =
            izip!(sectors, times, diffs).fold(LinearLayout::vertical(), |agg, line| {
                agg.child(
                    LinearLayout::horizontal()
                        .child(line.0)
                        .child(line.1)
                        .child(line.2),
                )
            });

        let total_time = Panel::new(total_time)
            .title("Final time")
            .title_position(HAlign::Left);

        let outer_layout = LinearLayout::vertical();
        outer_layout
            .child(
                Panel::new(sector_times)
                    .title("Sector times")
                    .title_position(HAlign::Left),
            )
            .child(total_time)
    }

    fn format_diff(diff: &i32) -> String {
        format!("{:+}", *diff as f32 / 1000.0)
    }

    /// Formats the passed duration as m:s.ms
    fn format_time(t: &Duration) -> String {
        let mut sec = t.as_secs();
        let sec = loop {
            if sec < 60 {
                break sec;
            } else {
                sec -= 60;
            }
        };
        format!("{}:{:02}.{}", t.as_secs() / 60, sec, t.subsec_millis())
    }

    /// Converts the absolute times from sectors to time spent in each sector.
    ///
    /// Returned vector contains the time for each sector in order, None for errored or incomplete sectors.
    pub fn get_sector_times(&self) -> Vec<Option<Duration>> {
        if self.state.is_not_started() {
            return self.sectors.iter().map(|_| None).collect();
        }

        // Start of a lap anchors the split times
        let start_time = match self.state {
            SplitState::Running(start) => start,
            SplitState::Completed(start, _) => start,
            _ => unreachable!(),
        };

        // Sector times are relative to the previous valid sectors stamp
        self.sectors
            .iter()
            .scan(start_time, |last_t, sect| match sect.state {
                SectorState::Invalidated | Incomplete => Some(None),
                SectorState::Complete(time) => {
                    let ret = Some(time.duration_since(*last_t).ok());
                    *last_t = time;
                    ret
                }
            })
            .collect()
    }

    /// Gets the total lap time. Returns None if not complete or if the end time is before the beginning.
    pub fn get_total_time(&self) -> Option<Duration> {
        match self.state {
            SplitState::NotStarted | SplitState::Running(_) => None,
            SplitState::Completed(start, end) => end.duration_since(start).ok(),
        }
    }

    /// Gets the time diffs for each sector compared to a previous run. That is, if a lap took 10s on last,
    /// and 11s in this run, the time in the vector will be +1000.
    ///
    /// Returned vector has diffs per sector in ms. Vector will be the length of the number of sectors in
    /// this lap. Any sectors without a time in either lap will be None.
    pub fn get_diffs(&self, last: &Self) -> Vec<Option<i32>> {
        let our_times = self.get_sector_times();
        let old_times = last.get_sector_times();

        // This will iterate until we hit the shorter of the two splits, it may have less sectors than this split
        let mut short_times: Vec<Option<i32>> = our_times
            .iter()
            .zip(old_times.iter())
            .map(|(tt, ot)| {
                if tt.is_some() && ot.is_some() {
                    Some(tt.unwrap().as_millis() as i32 - ot.unwrap().as_millis() as i32)
                } else {
                    None
                }
            })
            .collect();

        // Pad until we match the number of sectors in this split
        if short_times.len() < self.sectors.len() {
            let diff = self.sectors.len() - short_times.len();
            short_times.extend(std::iter::repeat(None).take(diff));
        }

        short_times
    }
}

/// State of the lap
#[derive(IsVariant, Debug, Unwrap, Clone)]
pub enum SplitState {
    /// Waiting for the first sensor to trigger
    NotStarted,
    /// Lap is occurring, started at time
    Running(SystemTime),
    /// Lap is done, started at left time, ended at right time
    Completed(SystemTime, SystemTime),
}

/// Single timing unit in a split, bounded by 2 sensors
#[derive(Debug, Eq, PartialEq, Clone)]
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
}

#[derive(Debug, Eq, PartialEq, IsVariant, Unwrap, Clone)]
enum SectorState {
    /// Sector was skipped
    Invalidated,
    /// Node has not been completed yet
    Incomplete,
    /// Sector was completed at the contained absolute time
    Complete(SystemTime),
}

#[cfg(test)]
mod tests {
    use crate::splits::{Sector, Splits};
    use std::collections::BTreeSet;
    use std::time::Duration;
    use timebay_common::messages::DetectionMessage;

    #[test]
    fn sectors_are_correct() {
        let splits = Splits::new(BTreeSet::from_iter(1u16..=3));

        assert_eq!(
            splits.sectors,
            vec![Sector::new(1, 2), Sector::new(2, 3), Sector::new(3, 1)]
        );

        // Out of order
        let splits = Splits::new(BTreeSet::from_iter([4, 3, 7, 2].into_iter()));

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
        let splits = Splits::new(BTreeSet::from_iter([1].into_iter()));

        assert_eq!(splits.sectors, vec![Sector::new(1, 1),]);
    }

    #[test]
    fn trigger_handling_works() {
        let mut splits = Splits::new(BTreeSet::from_iter(1u16..=2));

        assert!(splits.connect_node(3));

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

        assert!(!splits.connect_node(4));

        // Trigger the 3rd node, invalidating 1-2 and 2-3, leaving us in the last sector of 3-1
        assert!(splits
            .handle_node_trigger(DetectionMessage::new(3, 10, 2, 0))
            .is_running());
        assert!(splits.sectors[0].state.is_invalidated());
        assert!(splits.sectors[1].state.is_invalidated());
        assert!(splits.sectors[2].state.is_incomplete());
        assert_eq!(splits.current_sector, 2);

        // Trigger old node to ensure old nodes are still ignored even when on last node
        assert!(splits
            .handle_node_trigger(DetectionMessage::new(3, 10, 2, 0))
            .is_running());
        assert_eq!(splits.current_sector, 2);
        assert!(splits
            .handle_node_trigger(DetectionMessage::new(2, 10, 2, 0))
            .is_running());
        assert_eq!(splits.current_sector, 2);

        // Finish run
        assert!(splits
            .handle_node_trigger(DetectionMessage::new(1, 11, 3, 0))
            .is_completed());

        assert_eq!(splits.get_total_time().unwrap(), Duration::from_secs(2));

        assert_eq!(
            splits.get_sector_times(),
            vec![None, None, Some(Duration::from_secs(2))]
        );

        //TODO test single sensor
    }

    #[test]
    fn time_formatting() {
        let time = Duration::from_secs(62);
        assert_eq!(Splits::format_time(&time), String::from("1:02.0"));

        let time = Duration::from_secs(31) + Duration::from_millis(100);
        assert_eq!(Splits::format_time(&time), String::from("0:31.100"));
    }
}
