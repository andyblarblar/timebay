//! Lap timing logic and widgets

use crate::app::AppMessage;
use crate::splits::SectorState::Incomplete;
use derive_more::{IsVariant, Unwrap};
use iced::widget::{row, Column, Rule, Text};
use iced::Element;
use std::collections::BTreeSet;
use std::time::{Duration, SystemTime};
use timebay_common::messages::DetectionMessage;

/// Formats the passed duration as m:s:ms
pub fn format_time(t: &Duration) -> String {
    format!("{}:{}:{}", t.as_secs() / 60, t.as_secs(), t.subsec_millis())
}

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
            let sectors = { //TODO make sector creation a function, and add a function that lets us connect a new node and recreate sectors if we haven't started the run yet (needed for first run)
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
        // If we skipped over a node, we need to invalidate passed sectors (handling edge case of last sector, where node id is descending)
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
        // If a passed node triggered again, just ignore (accept edge case if it was rejected above, since node ordering is reversed there)
        else if self.sectors[self.current_sector as usize].nodes.1 > msg.node_id
            || (self.sectors[self.current_sector as usize].nodes.1 < msg.node_id
                && self.current_sector as usize == self.sectors.len() - 1)
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

    /// Creates the view for this set of splits.
    ///
    /// The last lap can be passed to generate time diffs.
    pub fn view(&self, last_lap: Option<Self>) -> Element<AppMessage> {
        let mut sectors = vec![];
        let mut times = vec![];
        let mut diffs = vec![];

        let splits = self.get_sector_times();
        let diffs_t = last_lap.map(|l| self.get_diffs(&l));

        for (i, sector) in self.sectors.iter().enumerate() {
            // Add sector name
            let name = format!("Sector {}-{}", sector.nodes.0, sector.nodes.1);
            sectors.push(Text::new(name).into());

            // Add sector time if done
            match sector.state {
                SectorState::Invalidated => {
                    times.push(
                        Text::new("INVALIDATED")
                            .style(iced::Color::from_rgb8(255, 0, 0))
                            .into(),
                    );
                    diffs.push(Text::new("N/A").into());
                }
                Incomplete => {
                    times.push(Text::new("").into());
                    diffs.push(Text::new("").into());
                }
                SectorState::Complete(_) => {
                    let sect_t = splits[i].unwrap();

                    times.push(Text::new(format_time(&sect_t)).into());

                    // Add diff time if diff is possible
                    if let Some(ref last_lap) = diffs_t {
                        if let Some(Some(diff)) = last_lap.get(i) {
                            diffs.push(
                                Text::new(diff.to_string())
                                    .style(if *diff < 0 {
                                        iced::Color::from_rgb8(0, 255, 0)
                                    } else if *diff > 0 {
                                        iced::Color::from_rgb8(255, 0, 0)
                                    } else {
                                        iced::Color::from_rgb8(0, 0, 0)
                                    })
                                    .into(),
                            );
                        }
                    } else {
                        diffs.push(Text::new("N/A").into());
                    }
                }
            }
        }

        let total_time = Text::new(
            self.get_total_time()
                .map(|t| format_time(&t))
                .unwrap_or(String::from("0:0:0")),
        );

        iced::widget::column![
            row![
                Column::with_children(sectors),
                Rule::vertical(5),
                Column::with_children(times),
                Rule::vertical(5),
                Column::with_children(diffs)
            ],
            row![total_time]
        ]
        .into()
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

/// State of the splits run
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
}

#[derive(Debug, Eq, PartialEq, IsVariant, Unwrap)]
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
    }
}
