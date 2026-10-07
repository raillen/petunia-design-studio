import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: leftToolbar
    width: 42
    Layout.preferredWidth: 42
    Layout.fillHeight: true

    property color bgToolbar: "#202021"
    property color borderDark: "#181819"
    property color textSecondary: "#a1a1a6"
    property color textDim: "#6c6c70"

    property bool editingStroke: false

    signal requestMoreTools()
    signal requestPathBrushes()
    signal requestColourTab()
    signal toolSelected(string toolId)

    readonly property var vectorTools: [
        { id: "select", icon: Phosphor.cursor, label: "Move Tool", key: "V" },
        { id: "artboard", icon: Phosphor.frameCorners, label: "Artboard Tool", key: "A" },
        { id: "node", icon: Phosphor.bezierCurve, label: "Node Tool", key: "N" },
        { id: "corner", icon: Phosphor.cornersOut, label: "Corner Tool", key: "C" },
        { id: "pen", icon: Phosphor.penNib, label: "Pen Tool", key: "P" },
        { id: "pencil", icon: Phosphor.pencilSimple, label: "Pencil Tool", key: "B" },
        { id: "brush", icon: Phosphor.paintBrush, label: "Vector Brush Tool", key: "B" },
        { id: "fill", icon: Phosphor.paintBucket, label: "Fill Tool", key: "G" },
        { id: "transparency", icon: Phosphor.drop, label: "Transparency Tool", key: "Y" },
        { id: "rectangle", icon: Phosphor.rectangle, label: "Rectangle Tool", key: "M" },
        { id: "ellipse", icon: Phosphor.circle, label: "Ellipse Tool", key: "O" },
        { id: "triangle", icon: Phosphor.triangle, label: "Triangle Tool", key: "" },
        { id: "star", icon: Phosphor.star, label: "Star Tool", key: "" },
        { id: "polygon", icon: Phosphor.polygon, label: "Polygon Tool", key: "" },
        { id: "diamond", icon: Phosphor.diamond, label: "Diamond Tool", key: "" },
        { id: "arrow", icon: Phosphor.arrowRight, label: "Arrow Tool", key: "" },
        { id: "heart", icon: Phosphor.heart, label: "Heart Tool", key: "" },
        { id: "cog", icon: Phosphor.gear, label: "Cog Tool", key: "" },
        { id: "text", icon: Phosphor.textT, label: "Frame Text Tool", key: "T" },
        { id: "crop", icon: Phosphor.crop, label: "Vector Crop Tool", key: "X" },
        { id: "knife", icon: Phosphor.knife, label: "Knife Tool", key: "K" },
        { id: "place", icon: Phosphor.image, label: "Place Image Tool", key: "" },
        { id: "shapebuilder", icon: Phosphor.shapes, label: "Shape Builder Tool", key: "" },
        { id: "measure", icon: Phosphor.ruler, label: "Measure Tool", key: "" },
        { id: "eyedropper", icon: Phosphor.eyedropper, label: "Color Picker Tool", key: "I" },
        { id: "hand", icon: Phosphor.hand, label: "View Tool (Hand)", key: "H" },
        { id: "zoom", icon: Phosphor.magnifyingGlass, label: "Zoom Tool", key: "Z" },
        { id: "more", icon: Phosphor.dotsThree, label: "More Tools", key: "" }
    ]

    readonly property var pixelTools: [
        { id: "select", icon: Phosphor.cursor, label: "Move Tool", key: "V" },
        { id: "pixel_select", icon: Phosphor.selection, label: "Pixel Marquee Tool", key: "M" },
        { id: "pixel_lasso", icon: Phosphor.lasso, label: "Pixel Lasso Tool", key: "L" },
        { id: "pixel_brush", icon: Phosphor.paintBrush, label: "Pixel Brush Tool", key: "B" },
        { id: "pixel_pencil", icon: Phosphor.pencilSimple, label: "Pixel Pencil Tool", key: "" },
        { id: "pixel_eraser", icon: Phosphor.eraser, label: "Eraser Tool", key: "E" },
        { id: "pixel_clone", icon: Phosphor.stamp, label: "Clone Stamp Tool", key: "S" },
        { id: "pixel_dodge", icon: Phosphor.sun, label: "Dodge Tool", key: "O" },
        { id: "pixel_burn", icon: Phosphor.moon, label: "Burn Tool", key: "" },
        { id: "pixel_smudge", icon: Phosphor.handPointing, label: "Smudge Tool", key: "" },
        { id: "pixel_blur", icon: Phosphor.drop, label: "Blur Tool", key: "" },
        { id: "pixel_fill", icon: Phosphor.paintBucket, label: "Flood Fill Tool", key: "G" },
        { id: "eyedropper", icon: Phosphor.eyedropper, label: "Color Picker Tool", key: "I" },
        { id: "hand", icon: Phosphor.hand, label: "View Tool (Hand)", key: "H" },
        { id: "zoom", icon: Phosphor.magnifyingGlass, label: "Zoom Tool", key: "Z" }
    ]

    readonly property var exportTools: [
        { id: "slice", icon: Phosphor.knife, label: "Slice Tool", key: "S" },
        { id: "slice_select", icon: Phosphor.cursor, label: "Slice Select Tool", key: "V" },
        { id: "hand", icon: Phosphor.hand, label: "View Tool (Hand)", key: "H" },
        { id: "zoom", icon: Phosphor.magnifyingGlass, label: "Zoom Tool", key: "Z" }
    ]

    color: bgToolbar
    border.color: borderDark
    border.width: 1

    ColumnLayout {
        anchors.fill: parent
        anchors.topMargin: 4
        anchors.bottomMargin: 8
        spacing: 2

        ScrollView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            ScrollBar.vertical.policy: ScrollBar.AlwaysOff

            Column {
                width: parent.width
                spacing: 1

                Repeater {
                    model: (bridge && bridge.persona === "pixel") ? leftToolbar.pixelTools : ((bridge && bridge.persona === "export") ? leftToolbar.exportTools : leftToolbar.vectorTools)

                    delegate: Rectangle {
                        width: 38
                        height: 26
                        anchors.horizontalCenter: parent.horizontalCenter
                        radius: 4
                        color: {
                            if (bridge && bridge.tool === modelData.id) return "#333336"
                            if (hoverArea.containsMouse) return "#2a2a2d"
                            return "transparent"
                        }

                        Text {
                            anchors.centerIn: parent
                            text: modelData.icon
                            color: (bridge && bridge.tool === modelData.id) ? "#ffffff" : leftToolbar.textSecondary
                            font.family: "Phosphor"
                            font.pixelSize: 15
                        }

                        MouseArea {
                            id: hoverArea
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                leftToolbar.toolSelected(modelData.id)
                                if (modelData.id === "shapebuilder") {
                                    if (bridge) bridge.shapeBuilderStart()
                                } else if (modelData.id === "more") {
                                    leftToolbar.requestPathBrushes()
                                } else {
                                    if (bridge) bridge.setTool(modelData.id)
                                }
                            }
                        }

                        ToolTip.visible: hoverArea.containsMouse
                        ToolTip.text: modelData.label + (modelData.key ? " (" + modelData.key + ")" : "")
                    }
                }
            }
        }

        // Bottom Left Color Wells (Fill Circle + Stroke Ring)
        Item {
            Layout.fillWidth: true
            Layout.preferredHeight: 38

            // Stroke Ring (Behind / Secondary)
            Rectangle {
                x: 17
                y: 11
                width: 18
                height: 18
                radius: 9
                color: "transparent"
                border.color: bridge ? bridge.currentStroke : "#000000"
                border.width: 2.5
                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        leftToolbar.editingStroke = true
                        leftToolbar.requestColourTab()
                    }
                }
            }

            // Fill Circle (In Front / Primary)
            Rectangle {
                x: 7
                y: 5
                width: 20
                height: 20
                radius: 10
                color: bridge ? bridge.currentFill : "#ffffff"
                border.color: leftToolbar.editingStroke ? "#ffffff" : "#222224"
                border.width: 1.5
                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        leftToolbar.editingStroke = false
                        leftToolbar.requestColourTab()
                    }
                }
            }

            // Swap Arrow Button
            Text {
                x: 24
                y: 1
                text: Phosphor.arrowsLeftRight
                color: swapHover.containsMouse ? "#ffffff" : leftToolbar.textDim
                font.family: "Phosphor"
                font.pixelSize: 11
                MouseArea {
                    id: swapHover
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: if (bridge) bridge.swapColors()
                }
                ToolTip.visible: swapHover.containsMouse
                ToolTip.text: "Inverter Preenchimento e Contorno"
            }
        }
    }
}
