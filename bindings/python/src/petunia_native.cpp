#include <nanobind/nanobind.h>
#include <nanobind/stl/optional.h>
#include <nanobind/stl/string.h>
#include <nanobind/stl/string_view.h>
#include <nanobind/stl/tuple.h>

#include "petunia/core/command/command.hpp"
#include "petunia/core/document/document.hpp"
#include "petunia/core/error/error.hpp"
#include "petunia/core/ids/ids.hpp"
#include "petunia/core/math/types.hpp"

namespace nb = nanobind;
namespace pc = petunia::core;

NB_MODULE(petunia_native, m) {
    m.doc() = "Petunia native core (G001-G008 baseline).";

    nb::class_<pc::ObjectId>(m, "ObjectId")
        .def_static("generate", &pc::ObjectId::generate)
        .def_static("parse", &pc::ObjectId::parse)
        .def("str", &pc::ObjectId::str)
        .def("__eq__", [](const pc::ObjectId& a, const pc::ObjectId& b) { return a == b; })
        .def("__repr__", [](const pc::ObjectId& id) { return "ObjectId('" + id.str() + "')"; });

    nb::class_<pc::Rectd>(m, "Rectd")
        .def(nb::init<>())
        .def(nb::init<double, double, double, double>())
        .def_rw("x", &pc::Rectd::x)
        .def_rw("y", &pc::Rectd::y)
        .def_rw("width", &pc::Rectd::width)
        .def_rw("height", &pc::Rectd::height)
        .def("validate", &pc::Rectd::validate);

    nb::class_<pc::Affine2D>(m, "Affine2D")
        .def(nb::init<>())
        .def_static("translation", &pc::Affine2D::translation)
        .def_static("scale", &pc::Affine2D::scale)
        .def("inverted", &pc::Affine2D::inverted);

    nb::class_<pc::DocumentStore>(m, "DocumentStore")
        .def(nb::init<>())
        .def("add_surface", [](pc::DocumentStore& s, std::string name, double w, double h) {
            return s.add_surface(std::move(name), {w, h}).str();
        })
        .def("add_rectangle", [](pc::DocumentStore& s, std::string surface_id,
                                  std::string name, double x, double y, double w, double h) {
            pc::RectangleRecord r;
            r.name = std::move(name);
            r.frame = {x, y, w, h};
            return s.add_rectangle(pc::SurfaceId::parse(surface_id), std::nullopt, r).str();
        })
        .def("transform_rectangle", [](pc::DocumentStore& s, std::string object_id,
                                        double x, double y, double w, double h) {
            auto* rec = s.find(pc::ObjectId::parse(object_id));
            if (!rec) return false;
            pc::TransformRectangleCommand cmd(pc::ObjectId::parse(object_id), {x, y, w, h});
            pc::CommandContext ctx{s, {}};
            cmd.apply(ctx);
            return true;
        })
        .def("object_count", &pc::DocumentStore::object_count)
        .def("get_rectangle", [](pc::DocumentStore& s, std::string object_id) {
            auto* rec = s.find(pc::ObjectId::parse(object_id));
            if (!rec) throw pc::Error(pc::ErrorCode::NotFound, "object not found");
            auto& r = std::get<pc::RectangleRecord>(*rec);
            return std::tuple(r.frame.x, r.frame.y, r.frame.width, r.frame.height);
        });

    m.def("native_ping", [] { return 1; });
}
