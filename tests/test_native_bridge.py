import petunia_native as pn


def test_object_id_roundtrip() -> None:
    oid = pn.ObjectId.generate()
    assert pn.ObjectId.parse(oid.str()) == oid


def test_rect_validation_rejects_nan() -> None:
    r = pn.Rectd(0, 0, float("nan"), 10)
    try:
        r.validate()
    except Exception:
        return
    raise AssertionError("expected validation failure")


def test_document_store_surface_count() -> None:
    store = pn.DocumentStore()
    store.add_surface("S", 100.0, 100.0)
    assert store.object_count() == 0


def test_native_add_and_transform() -> None:
    store = pn.DocumentStore()
    sid = store.add_surface("S", 100.0, 100.0)
    oid = store.add_rectangle(sid, "R", 0.0, 0.0, 10.0, 10.0)
    assert store.object_count() == 1
    assert store.get_rectangle(oid) == (0.0, 0.0, 10.0, 10.0)
    assert store.transform_rectangle(oid, 5.0, 5.0, 20.0, 20.0) is True
    assert store.get_rectangle(oid) == (5.0, 5.0, 20.0, 20.0)
