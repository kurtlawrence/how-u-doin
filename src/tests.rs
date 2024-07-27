use super::*;
use crate::report::*;

fn init() {
    super::init(consumers::Noop(Duration::ZERO));
}

fn reset_rems(xs: Vec<Progress>) -> Vec<Progress> {
    xs.into_iter()
        .map(|mut x| {
            match &mut x.report.state {
                State::InProgress { remaining, .. } => *remaining = Some(1.),
                State::Completed { duration } => *duration = 1.,
                _ => (),
            }

            x.children = reset_rems(x.children);
            x
        })
        .collect()
}

#[test]
fn uninit_fetch() {
    disable();
    let f = fetch();
    assert_eq!(f, None);
}

#[test]
fn report_creation_smoke_test() {
    init();

    let a = new().label("a");
    let _ = new().label("b");
    let _ = new_with_parent(a.id()).label("c");
    let _ = new_root().label("d");

    let f = fetch();
    assert_eq!(
        f,
        Some(vec![
            Progress {
                report: Report {
                    label: "a".into(),
                    desc: "".into(),
                    state: State::InProgress {
                        len: None,
                        pos: 0,
                        bytes: false,
                        remaining: None,
                    },
                    accums: vec![]
                },
                children: vec![
                    Progress {
                        report: Report {
                            label: "b".into(),
                            desc: "".into(),
                            state: State::InProgress {
                                len: None,
                                pos: 0,
                                bytes: false,
                                remaining: None,
                            },
                            accums: vec![]
                        },
                        children: vec![]
                    },
                    Progress {
                        report: Report {
                            label: "c".into(),
                            desc: "".into(),
                            state: State::InProgress {
                                len: None,
                                pos: 0,
                                bytes: false,
                                remaining: None,
                            },
                            accums: vec![]
                        },
                        children: vec![]
                    }
                ]
            },
            Progress {
                report: Report {
                    label: "d".into(),
                    desc: "".into(),
                    state: State::InProgress {
                        len: None,
                        pos: 0,
                        bytes: false,
                        remaining: None,
                    },
                    accums: vec![]
                },

                children: vec![]
            }
        ])
    );
}

#[test]
fn tx_api() {
    init();

    let a = new().label("a").set_len(100).fmt_as_bytes(true);
    a.desc("foo bar");

    a.inc().inc().inc_by(4_u8).add_err("errrr");

    let f = reset_rems(fetch().unwrap());
    assert_eq!(
        f,
        vec![Progress {
            report: Report {
                label: "a".into(),
                desc: "foo bar".into(),
                state: State::InProgress {
                    len: Some(100),
                    pos: 6,
                    bytes: true,
                    remaining: Some(1.)
                },
                accums: vec![Message {
                    severity: Severity::Error,
                    msg: "errrr".into(),
                }]
            },
            children: vec![]
        }]
    );

    a.set_pos(50_u8).add_warn("war");

    let f = reset_rems(fetch().unwrap());
    assert_eq!(
        f,
        vec![Progress {
            report: Report {
                label: "a".into(),
                desc: "foo bar".into(),
                state: State::InProgress {
                    len: Some(100),
                    pos: 50,
                    bytes: true,
                    remaining: Some(1.)
                },
                accums: vec![
                    Message {
                        severity: Severity::Error,
                        msg: "errrr".into(),
                    },
                    Message {
                        severity: Severity::Warn,
                        msg: "war".into(),
                    }
                ]
            },
            children: vec![]
        }]
    );

    a.add_info("yo").finish();

    let f = reset_rems(fetch().unwrap());
    assert_eq!(
        f,
        vec![Progress {
            report: Report {
                label: "a".into(),
                desc: "foo bar".into(),
                state: State::Completed { duration: 1. },
                accums: vec![
                    Message {
                        severity: Severity::Error,
                        msg: "errrr".into(),
                    },
                    Message {
                        severity: Severity::Warn,
                        msg: "war".into(),
                    },
                    Message {
                        severity: Severity::Info,
                        msg: "yo".into(),
                    }
                ]
            },
            children: vec![]
        }]
    );

    a.close();

    let f = fetch();
    assert_eq!(f, Some(vec![]));
}

#[test]
fn cancel_test() {
    disable();

    cancel();
    assert_eq!(cancelled(), None);

    init();

    assert_eq!(cancelled(), Some(false));

    let a = new();

    assert_eq!(a.cancelled(), false);

    cancel();

    assert_eq!(a.cancelled(), true);
    assert_eq!(cancelled(), Some(true));
}

#[test]
fn reset_test() {
    init();

    let _ = new().label("a");
    let f = fetch().unwrap();
    assert_eq!(
        f,
        vec![Progress {
            report: Report {
                label: "a".into(),
                ..Report::default()
            },
            children: vec![]
        }]
    );

    reset();

    let f = fetch();
    assert_eq!(f, Some(vec![]));
}
