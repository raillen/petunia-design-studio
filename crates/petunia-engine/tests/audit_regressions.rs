#[cfg(test)]
mod tests {
    use petunia_core::*;
    use petunia_engine::{compile_document, headless_frame, DocumentRevision};
    use petunia_render::{backend::RenderOptions, software::SoftwareRenderer};
    use petunia_render_model::*;

    fn doc(path: VectorPath) -> (Document, ObjectId) {
        let mut d = Document::new("audit");
        let n = SceneNode::new_path("path", path, ParentRef::Page(d.scene.default_page()));
        let id = n.id;
        d.scene.insert_root(d.scene.default_page(), n).unwrap();
        (d, id)
    }
    fn snapshot(d: &Document) -> RenderSnapshot {
        let (s, warnings) = compile_document(d, DocumentRevision::GENESIS, RenderQuality::Export);
        assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");
        s
    }
    fn pixel(d: &Document, x: usize, y: usize) -> Vec<u8> {
        let frame = headless_frame(snapshot(d), 32, 32, 1.0);
        let (bytes, _) = SoftwareRenderer::new(1 << 20, 1 << 20)
            .render(&frame, &RenderOptions::default())
            .unwrap();
        bytes[(y * 32 + x) * 4..(y * 32 + x) * 4 + 4].to_vec()
    }
    #[test]
    fn baseline_black_rectangle() {
        let (d, _) = doc(VectorPath::rect(2., 2., 20., 20.));
        assert_eq!(pixel(&d, 10, 10), vec![0, 0, 0, 255]);
    }
    #[test]
    fn hidden_node_must_not_paint() {
        let (mut d, id) = doc(VectorPath::rect(2., 2., 20., 20.));
        d.scene.get_node_mut(id).unwrap().visible = false;
        assert_eq!(pixel(&d, 10, 10), vec![255, 255, 255, 255]);
    }
    #[test]
    fn evenodd_must_preserve_hole() {
        let mut p = VectorPath::rect(2., 2., 26., 26.);
        p.contours
            .extend(VectorPath::rect(8., 8., 12., 12.).contours);
        p.fill_rule = FillRule::EvenOdd;
        let (d, _) = doc(p);
        assert_eq!(pixel(&d, 12, 12), vec![255, 255, 255, 255]);
    }
    #[test]
    fn appearance_item_zero_opacity_must_not_paint() {
        let (mut d, id) = doc(VectorPath::rect(2., 2., 20., 20.));
        if let SceneItem::Path(p) = &mut d.scene.get_node_mut(id).unwrap().item {
            p.appearance.items[0].opacity = 0.;
        }
        assert_eq!(pixel(&d, 10, 10), vec![255, 255, 255, 255]);
    }
    #[test]
    fn second_fill_must_paint_above_first() {
        let (mut d, id) = doc(VectorPath::rect(2., 2., 20., 20.));
        let white = Paint::Solid(ColorSource::Value(ColorValue::Process(ProcessColor {
            value: ProcessColorValue::Rgb(Rgba {
                r: 1.,
                g: 1.,
                b: 1.,
                alpha: 1.,
            }),
            space: ColorSpaceRef::Builtin(BuiltinColorSpace::Srgb),
        })));
        if let SceneItem::Path(p) = &mut d.scene.get_node_mut(id).unwrap().item {
            p.appearance.items.push(
                AppearanceItem::new(true, 1., BlendMode::Normal, AppearanceKind::Fill(white))
                    .unwrap(),
            );
        }
        assert_eq!(pixel(&d, 10, 10), vec![255, 255, 255, 255]);
    }
    #[test]
    fn parent_transform_must_move_child() {
        let mut d = Document::new("group");
        let page = d.scene.default_page();
        let mut g = SceneNode::new_path("group", VectorPath::new(), ParentRef::Page(page));
        g.item = SceneItem::Group(vec![]);
        g.transform = Transform2D::translation(16., 0.);
        let gid = g.id;
        d.scene.insert_root(page, g).unwrap();
        let n = SceneNode::new_path(
            "child",
            VectorPath::rect(2., 2., 8., 8.),
            ParentRef::Object(gid),
        );
        d.scene.insert_child(gid, n, None).unwrap();
        assert_eq!(pixel(&d, 5, 5), vec![255, 255, 255, 255]);
        assert_eq!(pixel(&d, 21, 5), vec![0, 0, 0, 255]);
    }
    #[test]
    fn snapshot_must_keep_authorial_page_identity() {
        let (d, _) = doc(VectorPath::rect(2., 2., 20., 20.));
        assert_eq!(snapshot(&d).pages[0].page, d.scene.default_page());
    }
    #[test]
    fn snapshot_must_include_all_pages() {
        let (mut d, _) = doc(VectorPath::rect(2., 2., 20., 20.));
        let page = d.scene.create_page();
        d.pages
            .insert(Page::new(page, "Page 2", d.setup.default_page));
        let n = SceneNode::new_path(
            "second page",
            VectorPath::rect(2., 2., 20., 20.),
            ParentRef::Page(page),
        );
        d.scene.insert_root(page, n).unwrap();
        d.validate().unwrap();
        assert_eq!(snapshot(&d).pages.len(), 2);
    }
    #[test]
    fn malformed_wasm_sections_must_be_rejected() {
        use petunia_engine::plugins::*;
        let manifest = PluginManifest {
            plugin_id: "audit.plugin".into(),
            version: PluginVersion {
                major: 1,
                minor: 0,
                patch: 0,
            },
            host_api_version: HostApiVersion { major: 1, minor: 0 },
            entrypoints: vec!["run".into()],
            capabilities: vec![],
            permissions: vec![],
            metadata: PluginMetadata::default(),
        };
        let mut host = WasmPluginHost::new(&manifest, PluginLimits::default(), 10000).unwrap();
        let mut bytes = WASM_MAGIC.to_vec();
        bytes.extend([0xff, 0xff, 0xff]);
        assert!(
            host.load_module(&bytes).is_err(),
            "invalid section ID 255 accepted"
        );
    }
    fn commit_op(
        d: &mut Document,
        h: &mut petunia_engine::History,
        op: petunia_engine::DocumentOp,
    ) {
        use petunia_engine::*;
        let request = TransactionRequest {
            command_id: CommandId::new_v4(),
            operations: vec![op],
            merge_key: None,
        };
        let prepared = prepare_transaction(d, request, h.current_revision()).unwrap();
        h.commit(d, prepared, HistoryDescription::EditObjects)
            .unwrap();
    }
    #[test]
    fn new_branch_after_undo_must_stay_dirty_and_use_new_revision() {
        use petunia_engine::*;
        let (mut d, id) = doc(VectorPath::rect(2., 2., 20., 20.));
        let mut h = History::new(1 << 20, 1 << 20);
        commit_op(
            &mut d,
            &mut h,
            DocumentOp::SetVisibility {
                object: id,
                visible: false,
            },
        );
        let saved = h.current_revision();
        h.save_checkpoint();
        h.undo(&mut d).unwrap();
        commit_op(
            &mut d,
            &mut h,
            DocumentOp::SetTransform {
                object: id,
                transform: Transform2D::translation(5., 0.),
            },
        );
        assert!(
            h.is_dirty(),
            "different document state reported clean; revision reused {:?}",
            h.current_revision()
        );
        assert_ne!(h.current_revision(), saved);
    }
    #[test]
    fn page_root_must_be_deletable_via_transaction() {
        use petunia_engine::*;
        let (d, id) = doc(VectorPath::rect(2., 2., 20., 20.));
        let request = TransactionRequest {
            command_id: CommandId::new_v4(),
            operations: vec![DocumentOp::RemoveSubtree { root: id }],
            merge_key: None,
        };
        assert!(
            prepare_transaction(&d, request, DocumentRevision::GENESIS).is_ok(),
            "page root deletion refused"
        );
    }
    #[test]
    fn clip_binding_must_affect_compiled_pixels() {
        let (mut d, id) = doc(VectorPath::rect(2., 2., 26., 26.));
        let clip = SceneNode::new_path(
            "clip",
            VectorPath::rect(2., 2., 8., 8.),
            ParentRef::Page(d.scene.default_page()),
        );
        let source = clip.id;
        d.scene.insert_root(d.scene.default_page(), clip).unwrap();
        d.scene.get_node_mut(id).unwrap().clip =
            Some(ClipBinding::new(id, source, BindingSourceUse::BindingOnly).unwrap());
        d.validate().unwrap();
        assert_eq!(pixel(&d, 20, 20), vec![255, 255, 255, 255]);
    }
    #[test]
    fn supported_effect_must_change_snapshot_or_report_warning() {
        let (mut d, id) = doc(VectorPath::rect(2., 2., 20., 20.));
        let baseline = snapshot(&d).pages[0].primitives.clone();
        d.scene
            .get_node_mut(id)
            .unwrap()
            .geometry_effects
            .items
            .push(
                EffectInstance::new(
                    true,
                    1.,
                    BlendMode::Normal,
                    None,
                    GeometryEffect::Offset(OffsetParams {
                        distance: Vec2::new(4., 4.),
                    }),
                )
                .unwrap(),
            );
        let (s, warnings) = compile_document(&d, DocumentRevision::GENESIS, RenderQuality::Export);
        assert!(
            s.pages[0].primitives != baseline || !warnings.is_empty(),
            "effect silently discarded"
        );
    }
    #[test]
    fn history_budget_must_account_for_path_payload() {
        use petunia_engine::*;
        let (mut d, id) = doc(VectorPath::rect(2., 2., 20., 20.));
        let mut p = VectorPath::new();
        let mut c = Contour::new(false);
        for i in 0..10000 {
            c.push_node(PathNode::line(Point::new(i as f64, 0.), NodeKind::Cusp));
        }
        p.push_contour(c);
        let minimum_bytes = p.contours[0].nodes.len() * std::mem::size_of::<PathNode>();
        assert!(minimum_bytes > 1024);
        let mut h = History::new(1024, 1024);
        let request = TransactionRequest {
            command_id: CommandId::new_v4(),
            operations: vec![DocumentOp::ReplacePath {
                object: id,
                path: p,
            }],
            merge_key: None,
        };
        let prepared = prepare_transaction(&d, request, h.current_revision()).unwrap();
        assert!(
            h.commit(&mut d, prepared, HistoryDescription::EditObjects)
                .is_err(),
            "hard budget 1024 accepted at least {minimum_bytes} bytes of new path payload"
        );
    }
    #[test]
    fn fragment_dependency_closure_must_include_clip_source() {
        use petunia_engine::fragments::*;
        let (mut d, id) = doc(VectorPath::rect(2., 2., 26., 26.));
        let clip = SceneNode::new_path(
            "clip",
            VectorPath::rect(2., 2., 8., 8.),
            ParentRef::Page(d.scene.default_page()),
        );
        let source = clip.id;
        d.scene.insert_root(d.scene.default_page(), clip).unwrap();
        d.scene.get_node_mut(id).unwrap().clip =
            Some(ClipBinding::new(id, source, BindingSourceUse::BindingOnly).unwrap());
        d.validate().unwrap();
        let fragment = collect_fragment(
            &d,
            &[id],
            FragmentMetadata {
                source_document: d.id,
                source_revision: 0,
            },
        )
        .unwrap();
        assert!(
            fragment.nodes.iter().any(|n| n.id == source),
            "mandatory clip dependency excluded from copy fragment"
        );
    }
}
