import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components"

Item {
    id: transformStudio
    Layout.fillWidth: true
    Layout.fillHeight: true

    property color bgPanel: "#242426"
    property color bgField: "#181819"
    property color bgFieldHover: "#202022"
    property color borderSubtle: "#333335"
    property color borderFocus: "#00b4d8"
    property color textPrimary: "#f2f2f5"
    property color textSecondary: "#a1a1a6"
    property color textDim: "#6c6c70"
    property color accentCyan: "#00b4d8"

    property string anchorPoint: "center" // "nw", "n", "ne", "w", "center", "e", "sw", "s", "se"
    property bool keepAspectRatio: true

    function selectedObject() {
        var objs = bridge.objects
        for (var i = 0; i < objs.length; i++) {
            if (objs[i].id === bridge.selectedId) return objs[i]
        }
        return null
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 8
        spacing: 6

        // -------------------------------------------------------------
        // Row 1: Anchor Point (3x3 grid) + Quick Transform Buttons
        // -------------------------------------------------------------
        RowLayout {
            Layout.fillWidth: true
            spacing: 12

            // 3x3 Reference Point Anchor
            Rectangle {
                width: 38
                height: 38
                color: transformStudio.bgField
                border.color: transformStudio.borderSubtle
                radius: 3

                GridLayout {
                    anchors.centerIn: parent
                    columns: 3
                    rowSpacing: 4
                    columnSpacing: 4

                    Repeater {
                        model: [
                            { id: "nw" }, { id: "n" }, { id: "ne" },
                            { id: "w" },  { id: "center" }, { id: "e" },
                            { id: "sw" }, { id: "s" }, { id: "se" }
                        ]
                        delegate: Rectangle {
                            width: 6
                            height: 6
                            radius: 1
                            color: transformStudio.anchorPoint === modelData.id ? transformStudio.accentCyan : "#52525b"

                            MouseArea {
                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                onClicked: transformStudio.anchorPoint = modelData.id
                            }
                        }
                    }
                }
            }

            // Quick Flip & Rotate Buttons
            RowLayout {
                Layout.fillWidth: true
                spacing: 3

                // Rotate CCW (-90)
                Rectangle {
                    Layout.preferredWidth: 26
                    Layout.preferredHeight: 26
                    radius: 3
                    color: rotCcwMouse.containsMouse && bridge.selectedId ? "#333338" : transformStudio.bgField
                    border.color: transformStudio.borderSubtle

                    PIcon {
                        anchors.centerIn: parent
                        name: Phosphor.arrowCounterClockwise
                        size: 13
                        color: bridge.selectedId ? transformStudio.textPrimary : transformStudio.textDim
                    }

                    MouseArea {
                        id: rotCcwMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: bridge.selectedId ? Qt.PointingHandCursor : Qt.ArrowCursor
                        onClicked: if (bridge.selectedId) bridge.rotateSelected(-90)
                    }
                }

                // Rotate CW (+90)
                Rectangle {
                    Layout.preferredWidth: 26
                    Layout.preferredHeight: 26
                    radius: 3
                    color: rotCwMouse.containsMouse && bridge.selectedId ? "#333338" : transformStudio.bgField
                    border.color: transformStudio.borderSubtle

                    PIcon {
                        anchors.centerIn: parent
                        name: Phosphor.arrowClockwise
                        size: 13
                        color: bridge.selectedId ? transformStudio.textPrimary : transformStudio.textDim
                    }

                    MouseArea {
                        id: rotCwMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: bridge.selectedId ? Qt.PointingHandCursor : Qt.ArrowCursor
                        onClicked: if (bridge.selectedId) bridge.rotateSelected(90)
                    }
                }

                // Flip Horizontal
                Rectangle {
                    Layout.preferredWidth: 26
                    Layout.preferredHeight: 26
                    radius: 3
                    color: flipHMouse.containsMouse && bridge.selectedId ? "#333338" : transformStudio.bgField
                    border.color: transformStudio.borderSubtle

                    PIcon {
                        anchors.centerIn: parent
                        name: Phosphor.arrowsLeftRight
                        size: 13
                        color: bridge.selectedId ? transformStudio.textPrimary : transformStudio.textDim
                    }

                    MouseArea {
                        id: flipHMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: bridge.selectedId ? Qt.PointingHandCursor : Qt.ArrowCursor
                        onClicked: if (bridge.selectedId) bridge.flipSelected("horizontal")
                    }
                }

                // Flip Vertical
                Rectangle {
                    Layout.preferredWidth: 26
                    Layout.preferredHeight: 26
                    radius: 3
                    color: flipVMouse.containsMouse && bridge.selectedId ? "#333338" : transformStudio.bgField
                    border.color: transformStudio.borderSubtle

                    PIcon {
                        anchors.centerIn: parent
                        name: Phosphor.arrowsDownUp
                        size: 13
                        color: bridge.selectedId ? transformStudio.textPrimary : transformStudio.textDim
                    }

                    MouseArea {
                        id: flipVMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: bridge.selectedId ? Qt.PointingHandCursor : Qt.ArrowCursor
                        onClicked: if (bridge.selectedId) bridge.flipSelected("vertical")
                    }
                }
            }
        }

        // -------------------------------------------------------------
        // Row 2: X and Y Position
        // -------------------------------------------------------------
        RowLayout {
            Layout.fillWidth: true
            spacing: 6

            // X Field
            RowLayout {
                Layout.fillWidth: true
                spacing: 3
                Text { text: "X:"; color: transformStudio.textDim; font.pixelSize: 10; font.bold: true }
                TextField {
                    id: fieldX
                    Layout.fillWidth: true
                    font.pixelSize: 10
                    enabled: bridge.selectedId !== ""
                    text: transformStudio.selectedObject() ? Math.round(transformStudio.selectedObject().x).toString() : "—"
                    color: enabled ? transformStudio.textPrimary : transformStudio.textDim
                    background: Rectangle {
                        color: transformStudio.bgField
                        border.color: fieldX.activeFocus ? transformStudio.borderFocus : transformStudio.borderSubtle
                        radius: 2
                    }
                    onEditingFinished: if (transformStudio.selectedObject()) bridge.setProp("x", text)
                }
            }

            // Y Field
            RowLayout {
                Layout.fillWidth: true
                spacing: 3
                Text { text: "Y:"; color: transformStudio.textDim; font.pixelSize: 10; font.bold: true }
                TextField {
                    id: fieldY
                    Layout.fillWidth: true
                    font.pixelSize: 10
                    enabled: bridge.selectedId !== ""
                    text: transformStudio.selectedObject() ? Math.round(transformStudio.selectedObject().y).toString() : "—"
                    color: enabled ? transformStudio.textPrimary : transformStudio.textDim
                    background: Rectangle {
                        color: transformStudio.bgField
                        border.color: fieldY.activeFocus ? transformStudio.borderFocus : transformStudio.borderSubtle
                        radius: 2
                    }
                    onEditingFinished: if (transformStudio.selectedObject()) bridge.setProp("y", text)
                }
            }
        }

        // -------------------------------------------------------------
        // Row 3: W and H Size + Aspect Ratio Lock
        // -------------------------------------------------------------
        RowLayout {
            Layout.fillWidth: true
            spacing: 6

            // W Field
            RowLayout {
                Layout.fillWidth: true
                spacing: 3
                Text { text: "W:"; color: transformStudio.textDim; font.pixelSize: 10; font.bold: true }
                TextField {
                    id: fieldW
                    Layout.fillWidth: true
                    font.pixelSize: 10
                    enabled: bridge.selectedId !== ""
                    text: transformStudio.selectedObject() ? Math.round(transformStudio.selectedObject().width).toString() : "—"
                    color: enabled ? transformStudio.textPrimary : transformStudio.textDim
                    background: Rectangle {
                        color: transformStudio.bgField
                        border.color: fieldW.activeFocus ? transformStudio.borderFocus : transformStudio.borderSubtle
                        radius: 2
                    }
                    onEditingFinished: {
                        var o = transformStudio.selectedObject()
                        if (o) {
                            var newW = parseFloat(text) || o.width
                            if (transformStudio.keepAspectRatio && o.width > 0) {
                                var ratio = o.height / o.width
                                bridge.setProp("width", newW)
                                bridge.setProp("height", Math.round(newW * ratio))
                            } else {
                                bridge.setProp("width", newW)
                            }
                        }
                    }
                }
            }

            // Aspect Lock Toggle Button
            Rectangle {
                width: 18
                height: 18
                radius: 2
                color: aspectMouse.containsMouse ? "#333338" : "transparent"

                PIcon {
                    anchors.centerIn: parent
                    name: transformStudio.keepAspectRatio ? Phosphor.link : Phosphor.linkBreak
                    size: 11
                    color: transformStudio.keepAspectRatio ? transformStudio.accentCyan : transformStudio.textDim
                }

                MouseArea {
                    id: aspectMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: transformStudio.keepAspectRatio = !transformStudio.keepAspectRatio
                }
            }

            // H Field
            RowLayout {
                Layout.fillWidth: true
                spacing: 3
                Text { text: "H:"; color: transformStudio.textDim; font.pixelSize: 10; font.bold: true }
                TextField {
                    id: fieldH
                    Layout.fillWidth: true
                    font.pixelSize: 10
                    enabled: bridge.selectedId !== ""
                    text: transformStudio.selectedObject() ? Math.round(transformStudio.selectedObject().height).toString() : "—"
                    color: enabled ? transformStudio.textPrimary : transformStudio.textDim
                    background: Rectangle {
                        color: transformStudio.bgField
                        border.color: fieldH.activeFocus ? transformStudio.borderFocus : transformStudio.borderSubtle
                        radius: 2
                    }
                    onEditingFinished: {
                        var o = transformStudio.selectedObject()
                        if (o) {
                            var newH = parseFloat(text) || o.height
                            if (transformStudio.keepAspectRatio && o.height > 0) {
                                var ratio = o.width / o.height
                                bridge.setProp("height", newH)
                                bridge.setProp("width", Math.round(newH * ratio))
                            } else {
                                bridge.setProp("height", newH)
                            }
                        }
                    }
                }
            }
        }

        // -------------------------------------------------------------
        // Row 4: Rotation (R) and Shear (S)
        // -------------------------------------------------------------
        RowLayout {
            Layout.fillWidth: true
            spacing: 6

            // R Field
            RowLayout {
                Layout.fillWidth: true
                spacing: 3
                Text { text: "R:"; color: transformStudio.textDim; font.pixelSize: 10; font.bold: true }
                TextField {
                    id: fieldR
                    Layout.fillWidth: true
                    font.pixelSize: 10
                    enabled: bridge.selectedId !== ""
                    text: {
                        var o = transformStudio.selectedObject()
                        return o ? (Math.round(o.rotation || 0) + "°") : "—"
                    }
                    color: enabled ? transformStudio.textPrimary : transformStudio.textDim
                    background: Rectangle {
                        color: transformStudio.bgField
                        border.color: fieldR.activeFocus ? transformStudio.borderFocus : transformStudio.borderSubtle
                        radius: 2
                    }
                    onEditingFinished: {
                        var deg = parseFloat(text.replace("°", "")) || 0
                        if (bridge.selectedId) {
                            bridge.setProp("rotation", deg)
                        }
                    }
                }
            }

            // S Field (Shear)
            RowLayout {
                Layout.fillWidth: true
                spacing: 3
                Text { text: "S:"; color: transformStudio.textDim; font.pixelSize: 10; font.bold: true }
                TextField {
                    id: fieldS
                    Layout.fillWidth: true
                    font.pixelSize: 10
                    enabled: bridge.selectedId !== ""
                    text: {
                        var o = transformStudio.selectedObject()
                        return o ? (Math.round(o.shear || 0) + "°") : "—"
                    }
                    color: enabled ? transformStudio.textPrimary : transformStudio.textDim
                    background: Rectangle {
                        color: transformStudio.bgField
                        border.color: fieldS.activeFocus ? transformStudio.borderFocus : transformStudio.borderSubtle
                        radius: 2
                    }
                    onEditingFinished: {
                        var sh = parseFloat(text.replace("°", "")) || 0
                        if (bridge.selectedId) {
                            bridge.setProp("shear", sh)
                        }
                    }
                }
            }
        }

        Item { Layout.fillHeight: true }
    }
}
