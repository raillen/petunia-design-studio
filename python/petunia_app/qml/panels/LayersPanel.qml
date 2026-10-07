import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components"

Rectangle {
    id: layersPanel
    Layout.fillWidth: true
    Layout.fillHeight: true

    property color bgPanelBody: "#28282a"
    property color borderDark: "#181819"
    property color accentBlue: "#1976d2"
    property color textPrimary: "#f2f2f5"
    property color textSecondary: "#a1a1a6"
    property color textDim: "#6c6c70"

    color: bgPanelBody

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        // Layer header row (Opacity & Blend Mode)
        Rectangle {
            Layout.fillWidth: true
            height: 28
            color: "#202022"
            border.color: layersPanel.borderDark

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 6
                anchors.rightMargin: 6
                spacing: 4

                Text {
                    text: "Op:"
                    color: layersPanel.textDim
                    font.pixelSize: 10
                }

                PetuniaSlider {
                    id: opacitySlider
                    Layout.preferredWidth: 64
                    from: 0
                    to: 100
                    value: bridge ? Math.round(bridge.selectedOpacity * 100) : 100
                    onMoved: if (bridge) bridge.setOpacity(value / 100)
                }

                Text {
                    text: Math.round(opacitySlider.value) + "%"
                    color: layersPanel.textSecondary
                    font.pixelSize: 9
                    Layout.preferredWidth: 28
                }

                // Blend Mode ComboBox
                ComboBox {
                    id: blendCombo
                    Layout.fillWidth: true
                    Layout.preferredHeight: 20
                    model: [
                        "Normal", "Multiply", "Screen", "Overlay",
                        "Darken", "Lighten", "Color-Dodge", "Color-Burn",
                        "Hard-Light", "Soft-Light", "Difference", "Exclusion"
                    ]
                    currentIndex: {
                        var cur = (bridge && bridge.selectedBlendMode ? bridge.selectedBlendMode : "normal").toLowerCase()
                        for (var i = 0; i < model.length; i++) {
                            if (model[i].toLowerCase() === cur) return i
                        }
                        return 0
                    }
                    onActivated: function(index) {
                        var mode = model[index].toLowerCase()
                        if (bridge && bridge.selectedId) {
                            bridge.setBlendMode(bridge.selectedId, mode)
                        }
                    }
                    background: Rectangle {
                        color: "#181819"
                        border.color: "#333336"
                        radius: 2
                    }
                    contentItem: Text {
                        text: blendCombo.displayText
                        color: layersPanel.textPrimary
                        font.pixelSize: 9
                        verticalAlignment: Text.AlignVCenter
                        leftPadding: 4
                        elide: Text.ElideRight
                    }
                }

                // Lock indicator / toggle
                Text {
                    text: (bridge && bridge.selectedId && bridge.selectedIsLocked) ? Phosphor.lock : Phosphor.lockSimple
                    color: (bridge && bridge.selectedId && bridge.selectedIsLocked) ? "#38bdf8" : layersPanel.textDim
                    font.family: "Phosphor"
                    font.pixelSize: 12
                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: if (bridge && bridge.selectedId) bridge.toggleLock(bridge.selectedId)
                    }
                }
            }
        }

        // Layers Tree View
        ListView {
            id: layersList
            Layout.fillWidth: true
            Layout.fillHeight: true
            model: bridge ? bridge.layers : []
            clip: true

            delegate: Rectangle {
                width: ListView.view.width
                height: 24
                color: (bridge && bridge.selectedId === modelData.id) ? layersPanel.accentBlue : (layerMouse.containsMouse ? "#2f2f32" : "transparent")

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 4 + (modelData.depth * 12)
                    anchors.rightMargin: 0
                    spacing: 4

                    // Visibility checkbox / Eye
                    Text {
                        text: modelData.visible ? Phosphor.eye : Phosphor.eyeSlash
                        color: (bridge && bridge.selectedId === modelData.id) ? "#ffffff" : layersPanel.textDim
                        font.family: "Phosphor"
                        font.pixelSize: 11
                        MouseArea {
                            anchors.fill: parent
                            onClicked: if (bridge) bridge.toggleLayerVisibility(modelData.id)
                        }
                    }

                    // Expand / Collapse Arrow
                    Text {
                        text: modelData.childrenCount > 0 ? (modelData.expanded ? Phosphor.caretDown : Phosphor.caretRight) : " "
                        color: layersPanel.textSecondary
                        font.family: "Phosphor"
                        font.pixelSize: 10
                        Layout.preferredWidth: 8
                        MouseArea {
                            anchors.fill: parent
                            onClicked: if (bridge) bridge.toggleLayerExpanded(modelData.id)
                        }
                    }

                    // Layer type icon / Thumbnail
                    Rectangle {
                        width: 16
                        height: 14
                        radius: 2
                        clip: true
                        color: {
                            if (modelData.name.indexOf("Artboard") === 0) return "#f59e0b"
                            if (modelData.name.indexOf("Adjustment") !== -1) return "#3b82f6"
                            if (modelData.name.indexOf("Curves") !== -1) return "#6366f1"
                            if (modelData.name === "Pixel") return "#475569"
                            if (modelData.type === "group") return "#3b3b40"
                            if (modelData.type === "ellipse") return "#ec4899"
                            return "#6366f1"
                        }

                        Rectangle {
                            anchors.fill: parent
                            color: "#ffffff"
                            border.color: "#d1d5db"
                            visible: modelData.name.indexOf("Artboard") === 0
                            Text {
                                anchors.centerIn: parent
                                text: Phosphor.frameCorners
                                color: "#181819"
                                font.family: "Phosphor"
                                font.pixelSize: 9
                            }
                        }

                        Text {
                            anchors.centerIn: parent
                            visible: modelData.name.indexOf("Artboard") !== 0
                            text: {
                                if (modelData.name.indexOf("Adjustment") !== -1) return Phosphor.slidersHorizontal
                                if (modelData.name.indexOf("Curves") !== -1) return Phosphor.bezierCurve
                                if (modelData.name === "Pixel") return Phosphor.gridFour
                                if (modelData.type === "group") return Phosphor.folder
                                if (modelData.type === "ellipse") return Phosphor.circle
                                return Phosphor.rectangle
                            }
                            font.family: "Phosphor"
                            font.pixelSize: 9
                            color: "#ffffff"
                        }
                    }

                    // Layer Name
                    Text {
                        Layout.fillWidth: true
                        text: modelData.name
                        color: (bridge && bridge.selectedId === modelData.id) ? "#ffffff" : layersPanel.textPrimary
                        font.pixelSize: 11
                        font.bold: modelData.name.indexOf("Artboard") === 0 || modelData.type === "group"
                        elide: Text.ElideRight
                    }

                    // Lock icon
                    Text {
                        text: modelData.locked ? Phosphor.lock : ""
                        color: "#ffffff"
                        font.family: "Phosphor"
                        font.pixelSize: 9
                        visible: modelData.locked
                    }

                    // Color Tag Stripe on far right
                    Rectangle {
                        width: 6
                        Layout.fillHeight: true
                        color: modelData.colorTag !== "" ? modelData.colorTag : "transparent"
                    }
                }

                MouseArea {
                    id: layerMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: if (bridge) bridge.select(modelData.id)
                }
            }
        }

        // Layer Footer Actions
        Rectangle {
            Layout.fillWidth: true
            height: 24
            color: "#202022"
            border.color: layersPanel.borderDark

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 8
                anchors.rightMargin: 8
                spacing: 8

                Text {
                    text: Phosphor.caretDoubleUp
                    color: (bridge && bridge.selectedId) ? layersPanel.textSecondary : layersPanel.textDim
                    font.family: "Phosphor"; font.pixelSize: 11
                    ToolTip.visible: topMouse.containsMouse
                    ToolTip.text: "Trazer para a Frente (Ctrl+Shift+])"
                    MouseArea {
                        id: topMouse
                        anchors.fill: parent; hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: if (bridge) bridge.bringToFront("")
                    }
                }
                Text {
                    text: Phosphor.caretUp
                    color: (bridge && bridge.selectedId) ? layersPanel.textSecondary : layersPanel.textDim
                    font.family: "Phosphor"; font.pixelSize: 11
                    ToolTip.visible: upMouse.containsMouse
                    ToolTip.text: "Avançar um Nível (Ctrl+])"
                    MouseArea {
                        id: upMouse
                        anchors.fill: parent; hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: if (bridge) bridge.moveForward("")
                    }
                }
                Text {
                    text: Phosphor.caretDown
                    color: (bridge && bridge.selectedId) ? layersPanel.textSecondary : layersPanel.textDim
                    font.family: "Phosphor"; font.pixelSize: 11
                    ToolTip.visible: downMouse.containsMouse
                    ToolTip.text: "Recuar um Nível (Ctrl+[)"
                    MouseArea {
                        id: downMouse
                        anchors.fill: parent; hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: if (bridge) bridge.moveBackward("")
                    }
                }
                Text {
                    text: Phosphor.caretDoubleDown
                    color: (bridge && bridge.selectedId) ? layersPanel.textSecondary : layersPanel.textDim
                    font.family: "Phosphor"; font.pixelSize: 11
                    ToolTip.visible: bottomMouse.containsMouse
                    ToolTip.text: "Enviar para o Fundo (Ctrl+Shift+[)"
                    MouseArea {
                        id: bottomMouse
                        anchors.fill: parent; hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: if (bridge) bridge.sendToBack("")
                    }
                }

                Item { Layout.fillWidth: true }

                Text {
                    text: Phosphor.plus
                    color: layersPanel.textPrimary
                    font.family: "Phosphor"; font.pixelSize: 12; font.bold: true
                    ToolTip.visible: addMouse.containsMouse
                    ToolTip.text: "Criar Objeto Retângulo"
                    MouseArea {
                        id: addMouse
                        anchors.fill: parent; hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: if (bridge) bridge.create_shape("rectangle", 200, 200)
                    }
                }
                Text {
                    text: Phosphor.trash
                    color: (bridge && bridge.selectedId) ? "#ef4444" : layersPanel.textDim
                    font.family: "Phosphor"; font.pixelSize: 11
                    ToolTip.visible: delMouse.containsMouse
                    ToolTip.text: "Excluir Seleção (Delete)"
                    MouseArea {
                        id: delMouse
                        anchors.fill: parent; hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: if (bridge) bridge.deleteSelected()
                    }
                }
            }
        }
    }
}
