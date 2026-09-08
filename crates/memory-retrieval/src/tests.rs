use super::*;

    fn reference(kind: &str, id: u128) -> CognitiveRef {
        let uuid = uuid::Uuid::from_u128(id);
        match kind {
            "tag" => CognitiveRef::Tag(nous_core::TagId(uuid)),
            "memory" => CognitiveRef::Memory(nous_core::MemoryId(uuid)),
            _ => CognitiveRef::Anchor(nous_core::AnchorId(uuid)),
        }
    }

    fn graph() -> WaveGraphGeneration {
        let a = reference("tag", 1);
        let b = reference("tag", 2);
        let c = reference("memory", 3);
        WaveGraphGeneration::build(
            vec![
                WaveNode {
                    serving_id: 9,
                    reference: a.clone(),
                    node_kind: WaveNodeKind::Tag,
                    embedding_key: None,
                    posting_key: None,
                    intrinsic_residual_gain: None,
                },
                WaveNode {
                    serving_id: 4,
                    reference: b.clone(),
                    node_kind: WaveNodeKind::Tag,
                    embedding_key: None,
                    posting_key: None,
                    intrinsic_residual_gain: None,
                },
                WaveNode {
                    serving_id: 5,
                    reference: c.clone(),
                    node_kind: WaveNodeKind::Memory,
                    embedding_key: None,
                    posting_key: None,
                    intrinsic_residual_gain: None,
                },
            ],
            &[
                WaveEdgeEvidence {
                    from: a.clone(),
                    to: b.clone(),
                    support_class: "host_explicit".into(),
                    association_kind: "experiential".into(),
                    polarity: "positive".into(),
                    support_value: 1.0,
                    bridge_hint: true,
                },
                WaveEdgeEvidence {
                    from: b,
                    to: c,
                    support_class: "memory_evidence".into(),
                    association_kind: "structural".into(),
                    polarity: "positive".into(),
                    support_value: 2.0,
                    bridge_hint: false,
                },
            ],
            WaveConfig::default(),
        )
        .unwrap()
    }

    #[test]
    fn row_mass_is_bounded_and_bridge_competes_for_reserve() {
        let graph = graph();
        for node in 0..graph.nodes.len() as u32 {
            assert!(graph.outbound_mass(node) <= graph.config.outbound_budget + 1e-9);
        }
    }

    #[test]
    fn river_records_only_actual_flow_and_is_finite() {
        let graph = graph();
        let river = propagate(
            &graph,
            &[SourceSeed {
                node: 0,
                weight: 1.0,
                seed_family: "tag".into(),
                origin_cue: "test".into(),
                hop_zero: true,
            }],
        );
        assert!(river.edges.iter().all(|edge| edge.flow > 0.0));
        assert!(river.provenance.len() <= graph.nodes.len());
        assert!(river.max_hops <= 4);
    }

    #[test]
    fn unordered_trail_has_no_fabricated_path_contact() {
        let graph = graph();
        let river = propagate(
            &graph,
            &[SourceSeed {
                node: 0,
                weight: 1.0,
                seed_family: "tag".into(),
                origin_cue: "test".into(),
                hop_zero: true,
            }],
        );
        let trail = CandidateSemanticTrail {
            memory: reference("memory", 3),
            nodes: vec![0, 1],
            order: TrailOrder::Unordered,
            provenance: None,
        };
        let observation = trail_topology_observation(
            &trail,
            &river.source_field,
            &river.node_potential,
            &river.node_potential,
            &river,
        );
        assert_eq!(observation.edge_contact, 0.0);
        assert_eq!(observation.structural_score, 0.0);
    }

    #[test]
    fn epa_basis_is_bounded_and_observable() {
        let vectors = (0..8_u64)
            .map(|key| {
                (
                    key,
                    vec![
                        1.0 + key as f64,
                        0.5 * (key as f64 + 1.0),
                        (key % 3) as f64 + 0.25,
                    ],
                )
            })
            .collect::<Vec<_>>();
        let basis = build_epa_basis(&vectors, "test-space").expect("EPA basis");
        assert!(basis.basis.len() <= 64);
        assert!(basis.basis.iter().all(|axis| axis.len() == 3));
        let observation = observe_epa(&basis, &[0.5, 1.0, 0.25]).expect("EPA observation");
        assert_eq!(observation.axis_energy.len(), basis.basis.len());
        assert!((0.0..=1.0).contains(&observation.entropy));
    }
