import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components"

Rectangle {
    id: layersStudio
    Layout.fillWidth: true
    Layout.fillHeight: true

    property color bgPanelHeader: "#202021"
    property color bgPanelTabActive: "#2b2b2d"
    property color bgPanelTabInactive: "#1f1f20"
    property color bgPanelBody: "#28282a"
    property color borderDark: "#181819"
    property color accentBlue: "#1976d2"
    property color textPrimary: "#f2f2f5"
    property color textSecondary: "#a1a1a6"
    property color textDim: "#6c6c70"

    property int activeTab: 0 // 0: Layers, 1: Path Brushes, 2: Quick FX, 3: Styles

    color: bgPanelBody

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        // Tab Bar
        Rectangle {
            Layout.fillWidth: true
            height: 25
            color: layersStudio.bgPanelHeader

            RowLayout {
                anchors.fill: parent
                spacing: 0
                Repeater {
                    model: ["Layers", "Path Brushes", "Quick FX", "Styles"]
                    delegate: Rectangle {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        color: layersStudio.activeTab === index ? layersStudio.bgPanelTabActive : layersStudio.bgPanelTabInactive
                        Text {
                            anchors.centerIn: parent
                            text: modelData
                            color: layersStudio.activeTab === index ? "#ffffff" : layersStudio.textSecondary
                            font.pixelSize: 10
                            font.bold: layersStudio.activeTab === index
                        }
                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: layersStudio.activeTab = index
                        }
                    }
                }
            }
        }

        // Tab 0: Layers (Exact Affinity Designer Layer Tree)
        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: layersStudio.activeTab === 0

            ColumnLayout {
                anchors.fill: parent
                spacing: 0

                // Layer header row (Opacity & Blend Mode)
                Rectangle {
                    Layout.fillWidth: true
                    height: 28
                    color: "#202022"
                    border.color: layersStudio.borderDark

                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 6
                        anchors.rightMargin: 6
                        spacing: 4

                        Text {
                            text: "Op:"
                            color: layersStudio.textDim
                            font.pixelSize: 10
                        }

                        Slider {
                            id: opacitySlider
                            Layout.preferredWidth: 50
                            Layout.preferredHeight: 18
                            from: 0; to: 100
                            value: Math.round(bridge.selectedOpacity * 100)
                            onMoved: bridge.setOpacity(value / 100)
                        }

                        Text {
                            text: Math.round(opacitySlider.value) + "%"
                            color: layersStudio.textSecondary
                            font.pixelSize: 9
                            Layout.preferredWidth: 26
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
                                var cur = (bridge.selectedBlendMode || "normal").toLowerCase()
                                for (var i = 0; i < model.length; i++) {
                                    if (model[i].toLowerCase() === cur) return i
                                }
                                return 0
                            }
                            onActivated: function(index) {
                                var mode = model[index].toLowerCase()
                                if (bridge.selectedId) {
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
                                color: layersStudio.textPrimary
                                font.pixelSize: 9
                                verticalAlignment: Text.AlignVCenter
                                leftPadding: 4
                                elide: Text.ElideRight
                            }
                        }

                        // Lock indicator / toggle
                        Text {
                            text: bridge.selectedId && bridge.selectedIsLocked ? Phosphor.lock : Phosphor.lockSimple
                            color: bridge.selectedId && bridge.selectedIsLocked ? "#38bdf8" : layersStudio.textDim
                            font.family: "Phosphor"
                            font.pixelSize: 11
                            MouseArea {
                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                onClicked: if (bridge.selectedId) bridge.toggleLock(bridge.selectedId)
                            }
                        }
                    }
                }

                // Layers Tree View
                ListView {
                    id: layersList
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    model: bridge.layers
                    clip: true

                    delegate: Rectangle {
                        width: ListView.view.width
                        height: 24
                        color: bridge.selectedId === modelData.id ? layersStudio.accentBlue : (layerMouse.containsMouse ? "#2f2f32" : "transparent")

                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: 4 + (modelData.depth * 12)
                            anchors.rightMargin: 0
                            spacing: 4

                            // Visibility checkbox / Eye
                            Text {
                                text: modelData.visible ? Phosphor.eye : Phosphor.eyeSlash
                                color: bridge.selectedId === modelData.id ? "#ffffff" : layersStudio.textDim
                                font.family: "Phosphor"
                                font.pixelSize: 11
                                MouseArea {
                                    anchors.fill: parent
                                    onClicked: bridge.toggleLayerVisibility(modelData.id)
                                }
                            }

                            // Expand / Collapse Arrow
                            Text {
                                text: modelData.childrenCount > 0 ? (modelData.expanded ? Phosphor.caretDown : Phosphor.caretRight) : " "
                                color: layersStudio.textSecondary
                                font.family: "Phosphor"
                                font.pixelSize: 10
                                Layout.preferredWidth: 8
                                MouseArea {
                                    anchors.fill: parent
                                    onClicked: bridge.toggleLayerExpanded(modelData.id)
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
                                color: bridge.selectedId === modelData.id ? "#ffffff" : layersStudio.textPrimary
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
                            onClicked: bridge.select(modelData.id)
                        }
                    }
                }

                // Layer Footer Actions (Reorder buttons, new, delete)
                Rectangle {
                    Layout.fillWidth: true
                    height: 24
                    color: "#202022"
                    border.color: layersStudio.borderDark

                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 8
                        anchors.rightMargin: 8
                        spacing: 8

                        // Reorder buttons
                        Text {
                            text: Phosphor.caretDoubleUp
                            color: bridge.selectedId ? layersStudio.textSecondary : layersStudio.textDim
                            font.family: "Phosphor"; font.pixelSize: 11
                            ToolTip.visible: topMouse.containsMouse
                            ToolTip.text: "Trazer para a Frente (Ctrl+Shift+])"
                            MouseArea {
                                id: topMouse
                                anchors.fill: parent; hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: bridge.bringToFront("")
                            }
                        }
                        Text {
                            text: Phosphor.caretUp
                            color: bridge.selectedId ? layersStudio.textSecondary : layersStudio.textDim
                            font.family: "Phosphor"; font.pixelSize: 11
                            ToolTip.visible: upMouse.containsMouse
                            ToolTip.text: "Avançar um Nível (Ctrl+])"
                            MouseArea {
                                id: upMouse
                                anchors.fill: parent; hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: bridge.moveForward("")
                            }
                        }
                        Text {
                            text: Phosphor.caretDown
                            color: bridge.selectedId ? layersStudio.textSecondary : layersStudio.textDim
                            font.family: "Phosphor"; font.pixelSize: 11
                            ToolTip.visible: downMouse.containsMouse
                            ToolTip.text: "Recuar um Nível (Ctrl+[)"
                            MouseArea {
                                id: downMouse
                                anchors.fill: parent; hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: bridge.moveBackward("")
                            }
                        }
                        Text {
                            text: Phosphor.caretDoubleDown
                            color: bridge.selectedId ? layersStudio.textSecondary : layersStudio.textDim
                            font.family: "Phosphor"; font.pixelSize: 11
                            ToolTip.visible: bottomMouse.containsMouse
                            ToolTip.text: "Enviar para o Fundo (Ctrl+Shift+[)"
                            MouseArea {
                                id: bottomMouse
                                anchors.fill: parent; hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: bridge.sendToBack("")
                            }
                        }

                        Item { Layout.fillWidth: true }

                        Text {
                            text: Phosphor.plus
                            color: layersStudio.textPrimary
                            font.family: "Phosphor"; font.pixelSize: 12; font.bold: true
                            ToolTip.visible: addMouse.containsMouse
                            ToolTip.text: "Criar Objeto Retângulo"
                            MouseArea {
                                id: addMouse
                                anchors.fill: parent; hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: bridge.create_shape("rectangle", 200, 200)
                            }
                        }
                        Text {
                            text: Phosphor.trash
                            color: bridge.selectedId ? "#ef4444" : layersStudio.textDim
                            font.family: "Phosphor"; font.pixelSize: 11
                            ToolTip.visible: delMouse.containsMouse
                            ToolTip.text: "Excluir Seleção (Delete)"
                            MouseArea {
                                id: delMouse
                                anchors.fill: parent; hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: bridge.deleteSelected()
                            }
                        }
                    }
                }
            }
        }

        // Tab 1: Path Brushes
        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: layersStudio.activeTab === 1
            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 8
                Text { text: "Pincéis Vetoriais"; color: layersStudio.textPrimary; font.pixelSize: 11; font.bold: true }
                ListView {
                    Layout.fillWidth: true; Layout.fillHeight: true
                    model: ["Solid Round", "Calligraphy 5pt", "Inked Outline", "Textured Chalk", "Watercolor Edge"]
                    delegate: Rectangle {
                        width: ListView.view.width; height: 24; color: "transparent"
                        Text { anchors.verticalCenter: parent.verticalCenter; text: modelData; color: layersStudio.textSecondary; font.pixelSize: 10 }
                    }
                }
            }
        }

        // Tab 2: Quick FX
        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: layersStudio.activeTab === 2
            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 8
                Text { text: "Efeitos de Camada (Quick FX)"; color: layersStudio.textPrimary; font.pixelSize: 11; font.bold: true }
                Repeater {
                    model: ["Gaussian Blur", "Outer Shadow", "Outline", "Inner Glow", "3D Bevel"]
                    delegate: RowLayout {
                        CheckBox { text: modelData; contentItem: Text { text: modelData; color: layersStudio.textSecondary; font.pixelSize: 10 } }
                    }
                }
                Item { Layout.fillHeight: true }
            }
        }

        // Tab 3: Styles & Symbols
        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: layersStudio.activeTab === 3
            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 8
                spacing: 4

                Text { text: "Símbolos (" + bridge.symbols.length + ")"; color: layersStudio.textPrimary; font.pixelSize: 10; font.bold: true }
                ListView {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 64
                    model: bridge.symbols
                    clip: true
                    delegate: RowLayout {
                        width: ListView.view.width
                        Text { text: modelData.name; color: layersStudio.textSecondary; font.pixelSize: 9; Layout.fillWidth: true }
                        Button { text: "Inserir"; font.pixelSize: 8; onClicked: bridge.placeSymbol(modelData.id, 100, 100) }
                    }
                }
                Button {
                    text: "+ Criar Símbolo da Seleção"
                    font.pixelSize: 9
                    enabled: bridge.selectedId !== ""
                    onClicked: bridge.createSymbol("", "")
                }

                Text { text: "Estilos (" + bridge.styles.length + ")"; color: layersStudio.textPrimary; font.pixelSize: 10; font.bold: true }
                ListView {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 64
                    model: bridge.styles
                    clip: true
                    delegate: RowLayout {
                        width: ListView.view.width
                        Text { text: modelData.name; color: layersStudio.textSecondary; font.pixelSize: 9; Layout.fillWidth: true }
                        Button { text: "Aplicar"; font.pixelSize: 8; onClicked: bridge.applyStyle(modelData.id) }
                    }
                }
                Button {
                    text: "+ Criar Estilo da Seleção"
                    font.pixelSize: 9
                    enabled: bridge.selectedId !== ""
                    onClicked: bridge.createStyle("Novo Estilo", "object", "")
                }
                Item { Layout.fillHeight: true }
            }
        }
    }
}
