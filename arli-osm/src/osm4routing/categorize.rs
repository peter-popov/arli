use lazy_static::lazy_static;
use regex::Regex;

// UNKNOWN accessiblity
const UNKNOWN: i8 = -1;

// FOOT_FORBIDDEN that no pedestrian is allowed
const FOOT_FORBIDDEN: i8 = 0;
// FOOT_ALLOWED pedestrians are allowed in both directions
const FOOT_ALLOWED: i8 = 1;

// CAR_FORBIDDEN no car is allowed
const CAR_FORBIDDEN: i8 = 0;
// CAR_RESIDENTIAL http://wiki.openstreetmap.org/wiki/Tag:highway%3Dresidential
const CAR_RESIDENTIAL: i8 = 1;
// CAR_TERTIARY http://wiki.openstreetmap.org/wiki/Tag:highway%3Dtertiary
const CAR_TERTIARY: i8 = 2;
// CAR_SECONDARY http://wiki.openstreetmap.org/wiki/Tag:highway%3Dsecondary
const CAR_SECONDARY: i8 = 3;
// car_forward http://wiki.http://wiki.openstreetmap.org/wiki/Tag:highway%3Dprimary
const CAR_PRIMARY: i8 = 4;
// CAR_TRUNK http://wiki.openstreetmap.org/wiki/Tag:highway%3Dtrunk
const CAR_TRUNK: i8 = 5;
// CAR_MOTORWAY http://wiki.openstreetmap.org/wiki/Tag:highway%3Dmotorway
const CAR_MOTORWAY: i8 = 6;

// BIKE_FORBIDDEN BIKE_ can not use this edge
const BIKE_FORBIDDEN: i8 = 0;
// BIKE_ALLOWED means that it can be used by a BIKE_, but the traffic might be shared with CAR_s
const BIKE_ALLOWED: i8 = 2;
// BIKE_LANE is a narrow lane dedicated for BIKE_, without physical separation from other traffic
const BIKE_LANE: i8 = 3;
// BIKE_BUSWAY means that BIKE_s are allowed on the bus lane
const BIKE_BUSWAY: i8 = 4;
// BIKE_TRACK is a physically separated for any other traffic
const BIKE_TRACK: i8 = 5;

// Edgeself contains what mode can use the edge in each direction
#[derive(Clone, Copy, Default)]
pub struct EdgeProperties {
    pub foot: i8,
    pub car_forward: i8,
    pub car_backward: i8,
    pub bike_forward: i8,
    pub bike_backward: i8,
    pub speed_limit_km_h: u8,
}

impl EdgeProperties {
    pub fn default() -> EdgeProperties {
        EdgeProperties {
            foot: UNKNOWN,
            car_forward: UNKNOWN,
            car_backward: UNKNOWN,
            bike_forward: UNKNOWN,
            bike_backward: UNKNOWN,
            speed_limit_km_h: 50, // TODO: default value based on road-class and region settings
        }
    }

    // Normalize fills UNKNOWN fields
    pub fn normalize(&mut self) {
        if self.car_backward == UNKNOWN {
            self.car_backward = self.car_forward;
        }
        if self.bike_backward == UNKNOWN {
            self.bike_backward = self.bike_forward;
        }
        if self.car_forward == UNKNOWN {
            self.car_forward = CAR_FORBIDDEN;
        }
        if self.bike_forward == UNKNOWN {
            self.bike_forward = BIKE_FORBIDDEN;
        }
        if self.car_backward == UNKNOWN {
            self.car_backward = CAR_FORBIDDEN;
        }
        if self.bike_backward == UNKNOWN {
            self.bike_backward = BIKE_FORBIDDEN;
        }
        if self.foot == UNKNOWN {
            self.foot = FOOT_FORBIDDEN;
        }
    }

    // Accessible means that at least one mean of transportation can use it in one direction
    pub fn accessible(self) -> bool {
        self.bike_forward != BIKE_FORBIDDEN
            || self.bike_backward != BIKE_FORBIDDEN
            || self.car_forward != CAR_FORBIDDEN
            || self.car_backward != CAR_FORBIDDEN
            || self.foot != FOOT_FORBIDDEN
    }

    fn parse_max_speed(val: &str) -> Option<u8> {
        lazy_static! {
            static ref MAX_SPEED_RE: Regex =
                Regex::new(r"^(?P<value>\d+)\s*(?P<unit>.*)$").unwrap();
        }
        if let Some(captures) = MAX_SPEED_RE.captures(val) {
            let result = captures
                .name("value")
                .map(|m| m.as_str().parse::<u8>().unwrap());
            if captures
                .name("unit")
                .filter(|m| m.as_str() == "mph")
                .is_some()
            {
                return result.map(|value| (value as f32 * 1.6) as u8);
            }
            return result;
        }
        None
    }

    pub fn update(&mut self, key: &str, val: &str) {
        match key {
            "highway" => match val {
                "cycleway" | "path" | "footway" | "steps" | "pedestrian" => {
                    self.bike_forward = BIKE_TRACK;
                    self.foot = FOOT_ALLOWED;
                }
                "primary" | "primary_link" => {
                    self.car_forward = CAR_PRIMARY;
                    self.foot = FOOT_ALLOWED;
                    self.bike_forward = BIKE_ALLOWED;
                }
                "secondary" => {
                    self.car_forward = CAR_SECONDARY;
                    self.foot = FOOT_ALLOWED;
                    self.bike_forward = BIKE_ALLOWED;
                }
                "tertiary" => {
                    self.car_forward = CAR_TERTIARY;
                    self.foot = FOOT_ALLOWED;
                    self.bike_forward = BIKE_ALLOWED;
                }
                "unclassified" | "residential" | "living_street" | "road" | "service" | "track" => {
                    self.car_forward = CAR_RESIDENTIAL;
                    self.foot = FOOT_ALLOWED;
                    self.bike_forward = BIKE_ALLOWED;
                }
                "motorway" | "motorway_link" => {
                    self.car_forward = CAR_MOTORWAY;
                    self.foot = FOOT_FORBIDDEN;
                    self.bike_forward = BIKE_FORBIDDEN;
                }
                "trunk" | "trunk_link" => {
                    self.car_forward = CAR_TRUNK;
                    self.foot = FOOT_FORBIDDEN;
                    self.bike_forward = BIKE_FORBIDDEN;
                }
                _ => {}
            },
            "pedestrian" | "foot" => match val {
                "no" => self.foot = FOOT_FORBIDDEN,
                _ => self.foot = FOOT_ALLOWED,
            },

            // http://wiki.openstreetmap.org/wiki/Cycleway
            // http://wiki.openstreetmap.org/wiki/Map_Features#Cycleway
            "cycleway" => match val {
                "track" => self.bike_forward = BIKE_TRACK,
                "opposite_track" => self.bike_backward = BIKE_TRACK,
                "opposite" => self.bike_backward = BIKE_ALLOWED,
                "share_busway" => self.bike_forward = BIKE_BUSWAY,
                "lane_left" | "opposite_lane" => self.bike_backward = BIKE_LANE,
                _ => self.bike_forward = BIKE_LANE,
            },

            "bicycle" => match val {
                "no" | "false" => self.bike_forward = BIKE_FORBIDDEN,
                _ => self.bike_forward = BIKE_ALLOWED,
            },
            "busway" => match val {
                "opposite_lane" | "opposite_track" => self.bike_backward = BIKE_BUSWAY,
                _ => self.bike_forward = BIKE_BUSWAY,
            },
            "oneway" => match val {
                "yes" | "true" | "1" => {
                    self.car_backward = CAR_FORBIDDEN;
                    if self.bike_backward == UNKNOWN {
                        self.bike_backward = BIKE_FORBIDDEN;
                    }
                }
                _ => {}
            },
            "junction" => {
                if val == "roundabout" {
                    self.car_backward = CAR_FORBIDDEN;
                    if self.bike_backward == UNKNOWN {
                        self.bike_backward = BIKE_FORBIDDEN;
                    }
                }
            }
            "maxspeed" => {
                self.speed_limit_km_h = Self::parse_max_speed(val).unwrap_or(self.speed_limit_km_h);
            }
            _ => {}
        }
    }
}

#[test]
fn test_accessible() {
    let mut p = EdgeProperties::default();
    p.normalize();
    assert!(!p.accessible());

    p.foot = FOOT_ALLOWED;
    assert!(p.accessible())
}

#[test]
fn test_normalize() {
    let mut p = EdgeProperties::default();
    p.bike_forward = BIKE_LANE;
    p.normalize();
    assert_eq!(BIKE_LANE, p.bike_backward);
    p.bike_forward = BIKE_ALLOWED;
    p.normalize();
    assert_eq!(BIKE_LANE, p.bike_backward);

    p.car_forward = CAR_SECONDARY;
    p.car_backward = UNKNOWN;
    p.normalize();
    assert_eq!(CAR_SECONDARY, p.car_backward)
}

#[test]
fn test_update() {
    let mut p = EdgeProperties::default();
    p.update("highway", "secondary");
    assert_eq!(CAR_SECONDARY, p.car_forward);

    p.update("highway", "primary_link");
    assert_eq!(CAR_PRIMARY, p.car_forward);

    p.update("highway", "motorway");
    assert_eq!(CAR_MOTORWAY, p.car_forward);

    p.update("highway", "residential");
    assert_eq!(CAR_RESIDENTIAL, p.car_forward);

    p.update("highway", "tertiary");
    assert_eq!(CAR_TERTIARY, p.car_forward);

    p.update("highway", "trunk");
    assert_eq!(CAR_TRUNK, p.car_forward);

    p.update("highway", "cycleway");
    assert_eq!(BIKE_TRACK, p.bike_forward);
    assert_eq!(FOOT_ALLOWED, p.foot);

    p.update("foot", "designated");
    assert_eq!(FOOT_ALLOWED, p.foot);

    p.update("foot", "no");
    assert_eq!(FOOT_FORBIDDEN, p.foot);

    p.update("cycleway", "lane");
    assert_eq!(BIKE_LANE, p.bike_forward);

    p.update("cycleway", "track");
    assert_eq!(BIKE_TRACK, p.bike_forward);

    p.update("cycleway", "opposite_lane");
    assert_eq!(BIKE_LANE, p.bike_backward);

    p.update("cycleway", "opposite_track");
    assert_eq!(BIKE_TRACK, p.bike_backward);

    p.update("cycleway", "opposite");
    assert_eq!(BIKE_ALLOWED, p.bike_backward);

    p.update("cycleway", "share_busway");
    assert_eq!(BIKE_BUSWAY, p.bike_forward);

    p.update("cycleway", "lane_left");
    assert_eq!(BIKE_LANE, p.bike_backward);

    p.update("bicycle", "yes");
    assert_eq!(BIKE_ALLOWED, p.bike_forward);

    p.update("bicycle", "no");
    assert_eq!(BIKE_FORBIDDEN, p.bike_forward);

    p.update("busway", "yes");
    assert_eq!(BIKE_BUSWAY, p.bike_forward);

    p.update("busway", "opposite_track");
    assert_eq!(BIKE_BUSWAY, p.bike_backward);

    p.update("oneway", "yes");
    assert_eq!(BIKE_FORBIDDEN, p.car_backward);
    assert!(p.bike_backward != BIKE_FORBIDDEN);

    p.bike_backward = UNKNOWN;
    p.update("oneway", "yes");
    assert_eq!(BIKE_FORBIDDEN, p.bike_backward);

    p.update("junction", "roundabout");
    assert_eq!(BIKE_FORBIDDEN, p.car_backward);

    p.bike_backward = UNKNOWN;
    p.update("junction", "roundabout");
    assert_eq!(BIKE_FORBIDDEN, p.bike_backward);
}

#[test]
fn test_speed_limit_re() {
    assert_eq!(EdgeProperties::parse_max_speed("40"), Some(40));
    assert_eq!(EdgeProperties::parse_max_speed("50 mph"), Some(80));
    assert_eq!(EdgeProperties::parse_max_speed("none"), None);
}
