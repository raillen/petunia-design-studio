import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: contextToolbar
    width: parent ? parent.width : 1280
    height: 32

    property color bgContext: "#282829"
    property color borderDark: "#181819"
    property color borderSubtle: "#333335"
    property color bgField: "#181819"
    property color accentCyan: "#00b4d8"
    property color accentBlue: "#1976d2"
    property color accentHover: "#388bfd"
    property color textPrimary: "#f2f2f5"
    property color textSecondary: "#a1a1a6"
    property color textDim: "#6c6c70"

    property string currentArtboardPreset: "fhd"
    property real artboardPresetW: 1920
    property real artboardPresetH: 1080
    property bool artboardIsPortrait: false

    property real measureDistance: 0.0
    property real measureAngle: 0.0

    signal requestDocumentSetup()
    signal requestAppSettings()
    signal requestPlaceImage()
    signal selectPreset(string key, real w, real h)
    signal toggleArtboardOrientation()

    function selectedObject() {
        if (!bridge || !bridge.objects) return null
        var objs = bridge.objects
        var selId = bridge.selectedId || ""
        for (var i = 0; i < objs.length; i++) {
            if (objs[i].id === selId) return objs[i]
        }
        return null
    }

    color: bgContext
    border.color: borderDark
    border.width: 1

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: 12
        anchors.rightMargin: 12

        // Move Tool Controls (Selection Info & Document Setup)
        RowLayout {
            visible: bridge && bridge.tool === "select"
            spacing: 8

            // Selection info text
            Text {
                text: {
                    if (bridge && bridge.selectedId !== "" && bridge.selectedName !== "Artboard1") {
                        var o = contextToolbar.selectedObject()
                        if (o) {
                            return o.name + " (" + Math.round(o.width) + " × " + Math.round(o.height) + " px)"
                        }
                        return bridge.selectedName
                    }
                    return "No Selection"
                }
                color: contextToolbar.textPrimary
                font.pixelSize: 11
                font.bold: bridge && bridge.selectedId !== "" && bridge.selectedName !== "Artboard1"
            }

            // Auto-select checkbox & target mode pills
            RowLayout {
                spacing: 6
                Rectangle {
                    width: 14
                    height: 14
                    radius: 3
                    color: (bridge && bridge.autoSelectEnabled) ? "#1976d2" : "#222225"
                    border.color: (bridge && bridge.autoSelectEnabled) ? contextToolbar.accentCyan : "#45454a"
                    border.width: 1

                    Text {
                        anchors.centerIn: parent
                        text: Phosphor.check
                        color: "#ffffff"
                        font.family: "Phosphor"
                        font.pixelSize: 10
                        font.bold: true
                        visible: bridge && bridge.autoSelectEnabled
                    }

                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: if (bridge) bridge.setAutoSelectEnabled(!bridge.autoSelectEnabled)
                    }
                }

                Text {
                    text: "Auto-select:"
                    color: contextToolbar.textSecondary
                    font.pixelSize: 11
                    verticalAlignment: Text.AlignVCenter
                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: if (bridge) bridge.setAutoSelectEnabled(!bridge.autoSelectEnabled)
                    }
                }

                // Mode: Cursor / Group / Layer
                Rectangle {
                    height: 20
                    Layout.preferredWidth: 64
                    radius: 4
                    color: "#202022"
                    border.color: "#38383a"
                    RowLayout {
                        anchors.fill: parent
                        spacing: 0
                        Rectangle {
                            Layout.fillWidth: true; Layout.fillHeight: true; color: "transparent"
                            Text { anchors.centerIn: parent; text: Phosphor.cursor; color: contextToolbar.textSecondary; font.family: "Phosphor"; font.pixelSize: 11 }
                            MouseArea { anchors.fill: parent; onClicked: bridge.setAutoSelectMode("default") }
                        }
                        Rectangle { width: 1; height: 12; color: "#38383a" }
                        Rectangle {
                            Layout.fillWidth: true; Layout.fillHeight: true; color: "transparent"
                            Text { anchors.centerIn: parent; text: Phosphor.rectangle; color: contextToolbar.textSecondary; font.family: "Phosphor"; font.pixelSize: 11 }
                            MouseArea { anchors.fill: parent; onClicked: bridge.setAutoSelectMode("group") }
                        }
                        Rectangle { width: 1; height: 12; color: "#38383a" }
                        Rectangle {
                            Layout.fillWidth: true; Layout.fillHeight: true; color: "transparent"
                            Text { anchors.centerIn: parent; text: Phosphor.stackSimple; color: contextToolbar.textSecondary; font.family: "Phosphor"; font.pixelSize: 11 }
                            MouseArea { anchors.fill: parent; onClicked: bridge.setAutoSelectMode("layer") }
                        }
                    }
                }
            }

            // Document Setup... button
            Rectangle {
                height: 22
                Layout.preferredWidth: 98
                radius: 4
                color: "#333336"
                border.color: "#444448"
                Text {
                    anchors.centerIn: parent
                    text: "Document Setup.."
                    color: contextToolbar.textPrimary
                    font.pixelSize: 10
                }
                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: contextToolbar.requestDocumentSetup()
                }
            }

            // App Settings... button
            Rectangle {
                height: 22
                Layout.preferredWidth: 80
                radius: 4
                color: "#333336"
                border.color: "#444448"
                Text {
                    anchors.centerIn: parent
                    text: "App Settings..."
                    color: contextToolbar.textPrimary
                    font.pixelSize: 10
                }
                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: contextToolbar.requestAppSettings()
                }
            }

            // Grid icon button
            Rectangle {
                width: 22; height: 22; radius: 3; color: "transparent"
                Text { anchors.centerIn: parent; text: Phosphor.gridFour; color: contextToolbar.textSecondary; font.family: "Phosphor"; font.pixelSize: 13 }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (bridge) bridge.toggleSnapping() }
            }

            // Gear / Preferences icon button
            Rectangle {
                width: 22; height: 22; radius: 3; color: "transparent"
                Text { anchors.centerIn: parent; text: Phosphor.gear; color: contextToolbar.textSecondary; font.family: "Phosphor"; font.pixelSize: 13 }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: contextToolbar.requestAppSettings() }
            }
        }

        Rectangle { width: 1; height: 16; color: "#38383a" }

        // Shape Builder Controls
        RowLayout {
            visible: bridge && bridge.shapeBuilderActive
            spacing: 4
            Text {
                text: "Shape Builder: selecione regiões"
                color: "#67e8f9"
                font.pixelSize: 10
                font.bold: true
            }
            Rectangle {
                height: 20; Layout.preferredWidth: 54; radius: 3; color: contextToolbar.accentBlue
                Text { anchors.centerIn: parent; text: "Aplicar"; color: "#ffffff"; font.pixelSize: 10; font.bold: true }
                MouseArea { anchors.fill: parent; onClicked: if (bridge) bridge.shapeBuilderCommit() }
            }
            Rectangle {
                height: 20; Layout.preferredWidth: 54; radius: 3; color: "#3a3a3d"
                Text { anchors.centerIn: parent; text: "Cancelar"; color: contextToolbar.textSecondary; font.pixelSize: 10 }
                MouseArea { anchors.fill: parent; onClicked: if (bridge) bridge.shapeBuilderCancel() }
            }
        }

        // Pen Controls
        RowLayout {
            visible: bridge && bridge.penActive
            spacing: 4
            Text {
                text: "Caneta: clique nos nós"
                color: "#fde047"
                font.pixelSize: 10
                font.bold: true
            }
            Rectangle {
                height: 20; Layout.preferredWidth: 54; radius: 3; color: contextToolbar.accentBlue
                Text { anchors.centerIn: parent; text: "Concluir"; color: "#ffffff"; font.pixelSize: 10; font.bold: true }
                MouseArea { anchors.fill: parent; onClicked: if (bridge) bridge.penFinish() }
            }
            Rectangle {
                height: 20; Layout.preferredWidth: 54; radius: 3; color: "#3a3a3d"
                Text { anchors.centerIn: parent; text: "Cancelar"; color: contextToolbar.textSecondary; font.pixelSize: 10 }
                MouseArea { anchors.fill: parent; onClicked: if (bridge) bridge.penCancel() }
            }
        }

        // Node Tool Controls
        RowLayout {
            visible: bridge && bridge.tool === "node" && bridge.selectedNode && bridge.selectedNode.objId !== undefined
            spacing: 4
            Rectangle {
                height: 20; Layout.preferredWidth: 64; radius: 3; color: "#3a3a3d"
                Text { anchors.centerIn: parent; text: "+ Inserir Nó"; color: contextToolbar.textPrimary; font.pixelSize: 9 }
                MouseArea {
                    anchors.fill: parent
                    onClicked: {
                        var n = bridge.selectedNode
                        bridge.insertNode(n.objId, n.contourIndex, 0, 0.5)
                    }
                }
            }
            Rectangle {
                height: 20; Layout.preferredWidth: 64; radius: 3; color: "#3a3a3d"
                Text { anchors.centerIn: parent; text: "- Excluir Nó"; color: contextToolbar.textPrimary; font.pixelSize: 9 }
                MouseArea {
                    anchors.fill: parent
                    onClicked: {
                        var n = bridge.selectedNode
                        bridge.deleteNode(n.objId, n.contourIndex, n.nodeIndex)
                    }
                }
            }
            Rectangle {
                height: 20; Layout.preferredWidth: 48; radius: 3; color: "#3a3a3d"
                Text { anchors.centerIn: parent; text: "Cúspide"; color: contextToolbar.textPrimary; font.pixelSize: 9 }
                MouseArea {
                    anchors.fill: parent
                    onClicked: {
                        var n = bridge.selectedNode
                        bridge.setNodeKind(n.objId, n.contourIndex, n.nodeIndex, "cusp")
                    }
                }
            }
            Rectangle {
                height: 20; Layout.preferredWidth: 46; radius: 3; color: "#3a3a3d"
                Text { anchors.centerIn: parent; text: "Suave"; color: contextToolbar.textPrimary; font.pixelSize: 9 }
                MouseArea {
                    anchors.fill: parent
                    onClicked: {
                        var n = bridge.selectedNode
                        bridge.setNodeKind(n.objId, n.contourIndex, n.nodeIndex, "smooth")
                    }
                }
            }
            Rectangle {
                height: 20; Layout.preferredWidth: 54; radius: 3; color: "#3a3a3d"
                Text { anchors.centerIn: parent; text: "Simétrico"; color: contextToolbar.textPrimary; font.pixelSize: 9 }
                MouseArea {
                    anchors.fill: parent
                    onClicked: {
                        var n = bridge.selectedNode
                        bridge.setNodeKind(n.objId, n.contourIndex, n.nodeIndex, "symmetric")
                    }
                }
            }
            Rectangle {
                height: 20; Layout.preferredWidth: 54; radius: 3; color: "#3a3a3d"
                Text { anchors.centerIn: parent; text: "Reverter"; color: contextToolbar.textPrimary; font.pixelSize: 9 }
                MouseArea {
                    anchors.fill: parent
                    onClicked: {
                        var n = bridge.selectedNode
                        bridge.reverseContour(n.objId, n.contourIndex)
                    }
                }
            }
            Rectangle {
                height: 20; Layout.preferredWidth: 54; radius: 3; color: "#3a3a3d"
                Text { anchors.centerIn: parent; text: "Quebrar"; color: contextToolbar.textPrimary; font.pixelSize: 9 }
                MouseArea {
                    anchors.fill: parent
                    onClicked: {
                        var n = bridge.selectedNode
                        bridge.breakContour(n.objId, n.contourIndex, n.nodeIndex)
                    }
                }
            }
        }

        // Selection Operations (Agrupar / Desagrupar / Duplicar)
        RowLayout {
            visible: bridge && bridge.tool === "select" && bridge.selectedIds && bridge.selectedIds.length > 0 && bridge.selectedName !== "Artboard1" && !bridge.shapeBuilderActive
            spacing: 4
            Rectangle {
                height: 20; Layout.preferredWidth: 52; radius: 3; color: "#333336"
                Text { anchors.centerIn: parent; text: "Agrupar"; color: contextToolbar.textPrimary; font.pixelSize: 9 }
                MouseArea { anchors.fill: parent; onClicked: if (bridge) bridge.groupSelected() }
            }
            Rectangle {
                height: 20; Layout.preferredWidth: 68; radius: 3; color: "#333336"
                Text { anchors.centerIn: parent; text: "Desagrupar"; color: contextToolbar.textPrimary; font.pixelSize: 9 }
                MouseArea { anchors.fill: parent; onClicked: if (bridge) bridge.ungroupSelected() }
            }
            Rectangle {
                height: 20; Layout.preferredWidth: 52; radius: 3; color: "#333336"
                Text { anchors.centerIn: parent; text: "Duplicar"; color: contextToolbar.textPrimary; font.pixelSize: 9 }
                MouseArea { anchors.fill: parent; onClicked: if (bridge) bridge.duplicateSelected() }
            }
            Rectangle {
                height: 20; Layout.preferredWidth: 78; radius: 3; color: "#333336"
                visible: bridge && bridge.selectedIds && bridge.selectedIds.length > 1
                Text { anchors.centerIn: parent; text: "Shape Builder"; color: contextToolbar.textPrimary; font.pixelSize: 9 }
                MouseArea { anchors.fill: parent; onClicked: if (bridge) bridge.shapeBuilderStart() }
            }
        }

        // Export Persona Context Controls
        RowLayout {
            visible: bridge && bridge.persona === "export"
            spacing: 6
            Text { text: "Export Persona:"; color: contextToolbar.accentCyan; font.pixelSize: 11; font.bold: true }
            Text { text: "Formato:"; color: contextToolbar.textSecondary; font.pixelSize: 10 }
            Rectangle {
                height: 20; Layout.preferredWidth: 44; radius: 3; color: "#3a3a3d"
                Text { anchors.centerIn: parent; text: "PNG"; color: "#ffffff"; font.pixelSize: 9; font.bold: true }
            }
            Rectangle {
                height: 20; Layout.preferredWidth: 44; radius: 3; color: "#252528"
                Text { anchors.centerIn: parent; text: "SVG"; color: contextToolbar.textSecondary; font.pixelSize: 9 }
            }
            Rectangle {
                height: 20; Layout.preferredWidth: 44; radius: 3; color: "#252528"
                Text { anchors.centerIn: parent; text: "PDF"; color: contextToolbar.textSecondary; font.pixelSize: 9 }
            }
            Rectangle { width: 1; height: 14; color: "#38383a" }
            Text { text: "Escala: 1x (100%)"; color: contextToolbar.textSecondary; font.pixelSize: 10 }
            Rectangle {
                height: 22; Layout.preferredWidth: 92; radius: 4; color: contextToolbar.accentBlue
                RowLayout {
                    anchors.centerIn: parent; spacing: 4
                    Text { text: Phosphor.fileArrowDown; color: "#ffffff"; font.family: "Phosphor"; font.pixelSize: 12 }
                    Text { text: "Exportar"; color: "#ffffff"; font.pixelSize: 10; font.bold: true }
                }
                MouseArea {
                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                    onClicked: if (bridge) bridge.exportPng("/tmp/petunia-design-export.png")
                }
            }
        }

        // Artboard Tool Context Controls
        RowLayout {
            visible: bridge && bridge.tool === "artboard"
            spacing: 6

            RowLayout {
                spacing: 4
                Text {
                    text: Phosphor.frameCorners
                    color: contextToolbar.accentCyan
                    font.family: "Phosphor"
                    font.pixelSize: 13
                }
                Text {
                    text: "Prancheta:"
                    color: contextToolbar.accentCyan
                    font.pixelSize: 11
                    font.bold: true
                }
            }

            // Preset Dropdown Button
            Rectangle {
                height: 22
                Layout.preferredWidth: 148
                radius: 3
                color: presetHover.containsMouse ? "#38383c" : "#2c2c2f"
                border.color: "#444448"

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 8
                    anchors.rightMargin: 8
                    spacing: 4

                    Text {
                        Layout.fillWidth: true
                        text: {
                            if (contextToolbar.currentArtboardPreset === "fhd") return "FHD (1920×1080)"
                            if (contextToolbar.currentArtboardPreset === "4k") return "4K UHD (3840×2160)"
                            if (contextToolbar.currentArtboardPreset === "square") return "Quadrado (1080×1080)"
                            if (contextToolbar.currentArtboardPreset === "story") return "Story (1080×1920)"
                            if (contextToolbar.currentArtboardPreset === "a4") return "A4 (2480×3508)"
                            if (contextToolbar.currentArtboardPreset === "mobile") return "Mobile (390×844)"
                            if (contextToolbar.currentArtboardPreset === "dribbble") return "Dribbble (1600×1200)"
                            if (contextToolbar.currentArtboardPreset === "twitter_header") return "Banner (1500×500)"
                            return "Tamanho do Doc"
                        }
                        color: contextToolbar.textPrimary
                        font.pixelSize: 10
                        elide: Text.ElideRight
                    }

                    Text {
                        text: Phosphor.caretDown
                        color: contextToolbar.textSecondary
                        font.family: "Phosphor"
                        font.pixelSize: 10
                    }
                }

                MouseArea {
                    id: presetHover
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: presetMenu.open()
                }

                Menu {
                    id: presetMenu
                    y: parent.height + 2

                    Action { text: "Tamanho do Documento"; onTriggered: contextToolbar.selectPreset("doc", bridge ? bridge.documentWidth : 1920, bridge ? bridge.documentHeight : 1080) }
                    Action { text: "FHD 1080p (1920 × 1080)"; onTriggered: contextToolbar.selectPreset("fhd", 1920, 1080) }
                    Action { text: "4K UHD (3840 × 2160)"; onTriggered: contextToolbar.selectPreset("4k", 3840, 2160) }
                    Action { text: "Quadrado Social (1080 × 1080)"; onTriggered: contextToolbar.selectPreset("square", 1080, 1080) }
                    Action { text: "Story / Reels (1080 × 1920)"; onTriggered: contextToolbar.selectPreset("story", 1080, 1920) }
                    Action { text: "A4 Impressão (2480 × 3508)"; onTriggered: contextToolbar.selectPreset("a4", 2480, 3508) }
                    Action { text: "Mobile iPhone (390 × 844)"; onTriggered: contextToolbar.selectPreset("mobile", 390, 844) }
                    Action { text: "Dribbble Shot (1600 × 1200)"; onTriggered: contextToolbar.selectPreset("dribbble", 1600, 1200) }
                    Action { text: "Twitter Header (1500 × 500)"; onTriggered: contextToolbar.selectPreset("twitter_header", 1500, 500) }
                }
            }

            // Orientation Toggle Buttons (Landscape / Portrait)
            RowLayout {
                spacing: 2

                Rectangle {
                    height: 22
                    width: 24
                    radius: 3
                    color: !contextToolbar.artboardIsPortrait ? contextToolbar.accentBlue : "#2c2c2f"
                    border.color: "#444448"
                    Text {
                        anchors.centerIn: parent
                        text: Phosphor.rectangle
                        color: "#ffffff"
                        font.family: "Phosphor"
                        font.pixelSize: 13
                    }
                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: if (contextToolbar.artboardIsPortrait) contextToolbar.toggleArtboardOrientation()
                    }
                }

                Rectangle {
                    height: 22
                    width: 24
                    radius: 3
                    color: contextToolbar.artboardIsPortrait ? contextToolbar.accentBlue : "#2c2c2f"
                    border.color: "#444448"
                    Text {
                        anchors.centerIn: parent
                        text: Phosphor.rectangle
                        rotation: 90
                        color: "#ffffff"
                        font.family: "Phosphor"
                        font.pixelSize: 13
                    }
                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: if (!contextToolbar.artboardIsPortrait) contextToolbar.toggleArtboardOrientation()
                    }
                }
            }

            // Dimension Readouts / Editors
            RowLayout {
                spacing: 4
                Text { text: "L:"; color: contextToolbar.textDim; font.pixelSize: 10 }
                TextField {
                    Layout.preferredWidth: 50
                    Layout.preferredHeight: 20
                    font.pixelSize: 10
                    text: Math.round(contextToolbar.artboardPresetW).toString()
                    color: "#ffffff"
                    background: Rectangle { color: contextToolbar.bgField; border.color: contextToolbar.borderSubtle; radius: 2 }
                    onEditingFinished: {
                        var v = parseFloat(text)
                        if (!isNaN(v) && v > 10) contextToolbar.artboardPresetW = v
                    }
                }

                Text { text: "A:"; color: contextToolbar.textDim; font.pixelSize: 10 }
                TextField {
                    Layout.preferredWidth: 50
                    Layout.preferredHeight: 20
                    font.pixelSize: 10
                    text: Math.round(contextToolbar.artboardPresetH).toString()
                    color: "#ffffff"
                    background: Rectangle { color: contextToolbar.bgField; border.color: contextToolbar.borderSubtle; radius: 2 }
                    onEditingFinished: {
                        var v = parseFloat(text)
                        if (!isNaN(v) && v > 10) contextToolbar.artboardPresetH = v
                    }
                }
            }

            // Insert Artboard Action Button
            Rectangle {
                height: 22
                Layout.preferredWidth: 124
                radius: 3
                color: contextToolbar.accentBlue
                border.color: contextToolbar.accentHover

                RowLayout {
                    anchors.centerIn: parent
                    spacing: 4
                    Text { text: "+"; color: "#ffffff"; font.pixelSize: 12; font.bold: true }
                    Text { text: "Inserir Prancheta"; color: "#ffffff"; font.pixelSize: 10; font.bold: true }
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        if (bridge) bridge.createArtboard("", 0, 0, contextToolbar.artboardPresetW, contextToolbar.artboardPresetH, "#ffffff")
                    }
                }
            }

            // Fit All Artboards Button
            Rectangle {
                height: 22
                Layout.preferredWidth: 84
                radius: 3
                color: "#333336"
                border.color: "#444448"
                Text {
                    anchors.centerIn: parent
                    text: "Ajustar Todas"
                    color: contextToolbar.textPrimary
                    font.pixelSize: 10
                }
                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: if (bridge) bridge.fitAllArtboards()
                }
            }
        }

        // Rectangle Tool Context Controls
        RowLayout {
            visible: bridge && bridge.tool === "rectangle"
            spacing: 6
            RowLayout {
                spacing: 4
                Text { text: Phosphor.rectangle; color: contextToolbar.accentCyan; font.family: "Phosphor"; font.pixelSize: 13 }
                Text { text: "Retângulo:"; color: contextToolbar.accentCyan; font.pixelSize: 11; font.bold: true }
            }
            Text { text: "Canto:"; color: contextToolbar.textSecondary; font.pixelSize: 10 }
            Repeater {
                model: [0, 8, 16, 24, 32]
                delegate: Rectangle {
                    height: 20; Layout.preferredWidth: 26; radius: 3
                    color: (bridge && bridge.selectedCornerRadius === modelData) ? contextToolbar.accentBlue : "#333336"
                    Text { anchors.centerIn: parent; text: modelData + ""; color: "#ffffff"; font.pixelSize: 9 }
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (bridge) bridge.setCornerRadius(bridge.selectedId, modelData) }
                }
            }
            Rectangle { width: 1; height: 14; color: "#38383a" }
            Rectangle {
                height: 20; Layout.preferredWidth: 105; radius: 3; color: "#333336"; border.color: "#444448"
                visible: bridge && bridge.selectedId !== ""
                Text { anchors.centerIn: parent; text: "Converter em Curvas"; color: contextToolbar.textPrimary; font.pixelSize: 9 }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (bridge) bridge.convertToCurves(bridge.selectedId) }
            }
        }

        // Ellipse Tool Context Controls
        RowLayout {
            visible: bridge && bridge.tool === "ellipse"
            spacing: 6
            RowLayout {
                spacing: 4
                Text { text: Phosphor.circle; color: contextToolbar.accentCyan; font.family: "Phosphor"; font.pixelSize: 13 }
                Text { text: "Elipse:"; color: contextToolbar.accentCyan; font.pixelSize: 11; font.bold: true }
            }
            Rectangle {
                height: 20; Layout.preferredWidth: 105; radius: 3; color: "#333336"; border.color: "#444448"
                visible: bridge && bridge.selectedId !== ""
                Text { anchors.centerIn: parent; text: "Converter em Curvas"; color: contextToolbar.textPrimary; font.pixelSize: 9 }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (bridge) bridge.convertToCurves(bridge.selectedId) }
            }
        }

        // Triangle Tool Context Controls
        RowLayout {
            visible: bridge && bridge.tool === "triangle"
            spacing: 6
            RowLayout {
                spacing: 4
                Text { text: Phosphor.triangle; color: contextToolbar.accentCyan; font.family: "Phosphor"; font.pixelSize: 13 }
                Text { text: "Triângulo:"; color: contextToolbar.accentCyan; font.pixelSize: 11; font.bold: true }
            }
            Text { text: "Forma Paramétrica"; color: contextToolbar.textDim; font.pixelSize: 9 }
            Rectangle { width: 1; height: 14; color: "#38383a" }
            Rectangle {
                height: 20; Layout.preferredWidth: 105; radius: 3; color: "#333336"; border.color: "#444448"
                visible: bridge && bridge.selectedId !== ""
                Text { anchors.centerIn: parent; text: "Converter em Curvas"; color: contextToolbar.textPrimary; font.pixelSize: 9 }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (bridge) bridge.convertToCurves(bridge.selectedId) }
            }
        }

        // Star Tool Context Controls
        RowLayout {
            visible: bridge && bridge.tool === "star"
            spacing: 6
            RowLayout {
                spacing: 4
                Text { text: Phosphor.star; color: contextToolbar.accentCyan; font.family: "Phosphor"; font.pixelSize: 13 }
                Text { text: "Estrela (" + (bridge ? bridge.starPoints : 5) + " Pontas):"; color: contextToolbar.accentCyan; font.pixelSize: 11; font.bold: true }
            }
            Text { text: "Pontas:"; color: contextToolbar.textSecondary; font.pixelSize: 9 }
            Repeater {
                model: [4, 5, 6, 8, 12]
                delegate: Rectangle {
                    height: 20; Layout.preferredWidth: 24; radius: 3
                    color: (bridge && bridge.starPoints === modelData) ? contextToolbar.accentBlue : "#333336"
                    Text { anchors.centerIn: parent; text: modelData + ""; color: "#ffffff"; font.pixelSize: 9 }
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (bridge) bridge.setStarPoints(modelData) }
                }
            }
            Rectangle { width: 1; height: 14; color: "#38383a" }
            Text { text: "Raio:"; color: contextToolbar.textSecondary; font.pixelSize: 9 }
            Repeater {
                model: [0.25, 0.45, 0.65]
                delegate: Rectangle {
                    height: 20; Layout.preferredWidth: 32; radius: 3
                    color: (bridge && Math.abs(bridge.starInnerRadius - modelData) < 0.05) ? contextToolbar.accentBlue : "#333336"
                    Text { anchors.centerIn: parent; text: Math.round(modelData * 100) + "%"; color: "#ffffff"; font.pixelSize: 8 }
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (bridge) bridge.setStarInnerRadius(modelData) }
                }
            }
            Rectangle { width: 1; height: 14; color: "#38383a" }
            Rectangle {
                height: 20; Layout.preferredWidth: 105; radius: 3; color: "#333336"; border.color: "#444448"
                visible: bridge && bridge.selectedId !== ""
                Text { anchors.centerIn: parent; text: "Converter em Curvas"; color: contextToolbar.textPrimary; font.pixelSize: 9 }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (bridge) bridge.convertToCurves(bridge.selectedId) }
            }
        }

        // Polygon Tool Context Controls
        RowLayout {
            visible: bridge && bridge.tool === "polygon"
            spacing: 6
            RowLayout {
                spacing: 4
                Text { text: Phosphor.polygon; color: contextToolbar.accentCyan; font.family: "Phosphor"; font.pixelSize: 13 }
                Text { text: "Polígono (" + (bridge ? bridge.polygonSides : 6) + " Lados):"; color: contextToolbar.accentCyan; font.pixelSize: 11; font.bold: true }
            }
            Text { text: "Lados:"; color: contextToolbar.textSecondary; font.pixelSize: 9 }
            Repeater {
                model: [3, 4, 5, 6, 8, 10, 12]
                delegate: Rectangle {
                    height: 20; Layout.preferredWidth: 24; radius: 3
                    color: (bridge && bridge.polygonSides === modelData) ? contextToolbar.accentBlue : "#333336"
                    Text { anchors.centerIn: parent; text: modelData + ""; color: "#ffffff"; font.pixelSize: 9 }
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (bridge) bridge.setPolygonSides(modelData) }
                }
            }
            Rectangle { width: 1; height: 14; color: "#38383a" }
            Rectangle {
                height: 20; Layout.preferredWidth: 105; radius: 3; color: "#333336"; border.color: "#444448"
                visible: bridge && bridge.selectedId !== ""
                Text { anchors.centerIn: parent; text: "Converter em Curvas"; color: contextToolbar.textPrimary; font.pixelSize: 9 }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (bridge) bridge.convertToCurves(bridge.selectedId) }
            }
        }

        // Diamond Tool Context Controls
        RowLayout {
            visible: bridge && bridge.tool === "diamond"
            spacing: 6
            RowLayout {
                spacing: 4
                Text { text: Phosphor.diamond; color: contextToolbar.accentCyan; font.family: "Phosphor"; font.pixelSize: 13 }
                Text { text: "Losango:"; color: contextToolbar.accentCyan; font.pixelSize: 11; font.bold: true }
            }
            Text { text: "Forma Paramétrica"; color: contextToolbar.textDim; font.pixelSize: 9 }
            Rectangle { width: 1; height: 14; color: "#38383a" }
            Rectangle {
                height: 20; Layout.preferredWidth: 105; radius: 3; color: "#333336"; border.color: "#444448"
                visible: bridge && bridge.selectedId !== ""
                Text { anchors.centerIn: parent; text: "Converter em Curvas"; color: contextToolbar.textPrimary; font.pixelSize: 9 }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (bridge) bridge.convertToCurves(bridge.selectedId) }
            }
        }

        // Arrow Tool Context Controls
        RowLayout {
            visible: bridge && bridge.tool === "arrow"
            spacing: 6
            RowLayout {
                spacing: 4
                Text { text: Phosphor.arrowRight; color: contextToolbar.accentCyan; font.family: "Phosphor"; font.pixelSize: 13 }
                Text { text: "Seta:"; color: contextToolbar.accentCyan; font.pixelSize: 11; font.bold: true }
            }
            Text { text: "Forma Paramétrica"; color: contextToolbar.textDim; font.pixelSize: 9 }
            Rectangle { width: 1; height: 14; color: "#38383a" }
            Rectangle {
                height: 20; Layout.preferredWidth: 105; radius: 3; color: "#333336"; border.color: "#444448"
                visible: bridge && bridge.selectedId !== ""
                Text { anchors.centerIn: parent; text: "Converter em Curvas"; color: contextToolbar.textPrimary; font.pixelSize: 9 }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (bridge) bridge.convertToCurves(bridge.selectedId) }
            }
        }

        // Heart Tool Context Controls
        RowLayout {
            visible: bridge && bridge.tool === "heart"
            spacing: 6
            RowLayout {
                spacing: 4
                Text { text: Phosphor.heart; color: contextToolbar.accentCyan; font.family: "Phosphor"; font.pixelSize: 13 }
                Text { text: "Coração:"; color: contextToolbar.accentCyan; font.pixelSize: 11; font.bold: true }
            }
            Text { text: "Curvas Bézier Suaves"; color: contextToolbar.textDim; font.pixelSize: 9 }
            Rectangle { width: 1; height: 14; color: "#38383a" }
            Rectangle {
                height: 20; Layout.preferredWidth: 105; radius: 3; color: "#333336"; border.color: "#444448"
                visible: bridge && bridge.selectedId !== ""
                Text { anchors.centerIn: parent; text: "Converter em Curvas"; color: contextToolbar.textPrimary; font.pixelSize: 9 }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (bridge) bridge.convertToCurves(bridge.selectedId) }
            }
        }

        // Cog Tool Context Controls
        RowLayout {
            visible: bridge && bridge.tool === "cog"
            spacing: 6
            RowLayout {
                spacing: 4
                Text { text: Phosphor.gear; color: contextToolbar.accentCyan; font.family: "Phosphor"; font.pixelSize: 13 }
                Text { text: "Engrenagem (" + (bridge ? bridge.cogTeeth : 8) + " Dentes):"; color: contextToolbar.accentCyan; font.pixelSize: 11; font.bold: true }
            }
            Text { text: "Dentes:"; color: contextToolbar.textSecondary; font.pixelSize: 9 }
            Repeater {
                model: [4, 6, 8, 12, 16, 24]
                delegate: Rectangle {
                    height: 20; Layout.preferredWidth: 24; radius: 3
                    color: (bridge && bridge.cogTeeth === modelData) ? contextToolbar.accentBlue : "#333336"
                    Text { anchors.centerIn: parent; text: modelData + ""; color: "#ffffff"; font.pixelSize: 9 }
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (bridge) bridge.setCogTeeth(modelData) }
                }
            }
            Rectangle { width: 1; height: 14; color: "#38383a" }
            Text { text: "Profundidade:"; color: contextToolbar.textSecondary; font.pixelSize: 9 }
            Repeater {
                model: [0.15, 0.25, 0.40]
                delegate: Rectangle {
                    height: 20; Layout.preferredWidth: 32; radius: 3
                    color: (bridge && Math.abs(bridge.cogToothDepth - modelData) < 0.05) ? contextToolbar.accentBlue : "#333336"
                    Text { anchors.centerIn: parent; text: Math.round(modelData * 100) + "%"; color: "#ffffff"; font.pixelSize: 8 }
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (bridge) bridge.setCogToothDepth(modelData) }
                }
            }
            Rectangle { width: 1; height: 14; color: "#38383a" }
            Text { text: "Furo:"; color: contextToolbar.textSecondary; font.pixelSize: 9 }
            Repeater {
                model: [0.0, 0.25, 0.40, 0.60]
                delegate: Rectangle {
                    height: 20; Layout.preferredWidth: 32; radius: 3
                    color: (bridge && Math.abs(bridge.cogHoleRadius - modelData) < 0.05) ? contextToolbar.accentBlue : "#333336"
                    Text { anchors.centerIn: parent; text: modelData === 0.0 ? "0%" : Math.round(modelData * 100) + "%"; color: "#ffffff"; font.pixelSize: 8 }
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (bridge) bridge.setCogHoleRadius(modelData) }
                }
            }
            Rectangle { width: 1; height: 14; color: "#38383a" }
            Rectangle {
                height: 20; Layout.preferredWidth: 105; radius: 3; color: "#333336"; border.color: "#444448"
                visible: bridge && bridge.selectedId !== ""
                Text { anchors.centerIn: parent; text: "Converter em Curvas"; color: contextToolbar.textPrimary; font.pixelSize: 9 }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (bridge) bridge.convertToCurves(bridge.selectedId) }
            }
        }

        // Pencil & Vector Brush Tool Context Controls
        RowLayout {
            visible: bridge && (bridge.tool === "pencil" || bridge.tool === "brush")
            spacing: 6
            RowLayout {
                spacing: 4
                Text {
                    text: (bridge && bridge.tool === "brush") ? Phosphor.paintBrush : Phosphor.pencilSimple
                    color: contextToolbar.accentCyan
                    font.family: "Phosphor"
                    font.pixelSize: 13
                }
                Text {
                    text: (bridge && bridge.tool === "brush") ? "Pincel Vetorial:" : "Lápis:"
                    color: contextToolbar.accentCyan
                    font.pixelSize: 11
                    font.bold: true
                }
            }
            Text { text: "Largura:"; color: contextToolbar.textSecondary; font.pixelSize: 10 }
            Repeater {
                model: [1, 2, 4, 8, 16]
                delegate: Rectangle {
                    height: 20; Layout.preferredWidth: 26; radius: 3
                    color: (bridge && bridge.currentStrokeWidth === modelData) ? contextToolbar.accentBlue : "#333336"
                    Text { anchors.centerIn: parent; text: modelData + "px"; color: "#ffffff"; font.pixelSize: 8 }
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (bridge) bridge.setStrokeWidth(modelData) }
                }
            }
            Rectangle { width: 1; height: 14; color: "#38383a" }
            Text { text: "Suavização: 40%"; color: contextToolbar.textSecondary; font.pixelSize: 9 }
        }

        // Corner Tool Context Controls
        RowLayout {
            visible: bridge && bridge.tool === "corner"
            spacing: 6
            RowLayout {
                spacing: 4
                Text { text: Phosphor.cornersOut
                    color: contextToolbar.accentCyan
                    font.family: "Phosphor"
                    font.pixelSize: 13
                }
                Text { text: "Ferramenta de Canto:"; color: contextToolbar.accentCyan; font.pixelSize: 11; font.bold: true }
            }
            Text { text: "Raio:"; color: contextToolbar.textSecondary; font.pixelSize: 10 }
            Repeater {
                model: [0, 6, 12, 18, 24, 36]
                delegate: Rectangle {
                    height: 20; Layout.preferredWidth: 28; radius: 3
                    color: (bridge && bridge.selectedCornerRadius === modelData) ? contextToolbar.accentBlue : "#333336"
                    Text { anchors.centerIn: parent; text: modelData + "px"; color: "#ffffff"; font.pixelSize: 9 }
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (bridge) bridge.setCornerRadius(bridge.selectedId, modelData) }
                }
            }
            Rectangle { width: 1; height: 14; color: "#38383a" }
            Rectangle {
                height: 20; Layout.preferredWidth: 105; radius: 3; color: "#333336"; border.color: "#444448"
                visible: bridge && bridge.selectedId !== ""
                Text { anchors.centerIn: parent; text: "Converter em Curvas"; color: contextToolbar.textPrimary; font.pixelSize: 9 }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (bridge) bridge.convertToCurves(bridge.selectedId) }
            }
        }

        // Fill / Gradient Tool Context Controls
        RowLayout {
            visible: bridge && bridge.tool === "fill"
            spacing: 6
            RowLayout {
                spacing: 4
                Text { text: Phosphor.paintBucket; color: contextToolbar.accentCyan; font.family: "Phosphor"; font.pixelSize: 13 }
                Text { text: "Preenchimento:"; color: contextToolbar.accentCyan; font.pixelSize: 11; font.bold: true }
            }
            Rectangle {
                height: 20; Layout.preferredWidth: 72; radius: 3; color: "#333336"; border.color: "#444448"
                Text { anchors.centerIn: parent; text: "+ Gradiente"; color: contextToolbar.textPrimary; font.pixelSize: 9 }
                MouseArea {
                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                    onClicked: if (bridge) bridge.setObjectGradient(bridge.selectedId, [bridge.currentFill, "#00b4d8"])
                }
            }
            Rectangle {
                height: 20; Layout.preferredWidth: 54; radius: 3; color: "#333336"; border.color: "#444448"
                visible: bridge && bridge.selectedGradient && bridge.selectedGradient.length > 0
                Text { anchors.centerIn: parent; text: "Inverter"; color: contextToolbar.textPrimary; font.pixelSize: 9 }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (bridge) bridge.reverseObjectGradient(bridge.selectedId) }
            }
            Rectangle {
                height: 20; Layout.preferredWidth: 54; radius: 3; color: "#333336"; border.color: "#444448"
                visible: bridge && bridge.selectedGradient && bridge.selectedGradient.length > 0
                Text { anchors.centerIn: parent; text: "Sólido"; color: contextToolbar.textPrimary; font.pixelSize: 9 }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (bridge) bridge.removeObjectGradient(bridge.selectedId) }
            }
        }

        // Transparency Tool Context Controls
        RowLayout {
            visible: bridge && bridge.tool === "transparency"
            spacing: 6
            RowLayout {
                spacing: 4
                Text { text: Phosphor.drop; color: contextToolbar.accentCyan; font.family: "Phosphor"; font.pixelSize: 13 }
                Text { text: "Transparência:"; color: contextToolbar.accentCyan; font.pixelSize: 11; font.bold: true }
            }
            Text { text: Math.round((bridge ? bridge.currentOpacity : 1.0) * 100) + "%"; color: contextToolbar.textPrimary; font.pixelSize: 10; font.bold: true }
            Repeater {
                model: [0.25, 0.50, 0.75, 1.0]
                delegate: Rectangle {
                    height: 20; Layout.preferredWidth: 34; radius: 3
                    color: (bridge && Math.abs(bridge.currentOpacity - modelData) < 0.05) ? contextToolbar.accentBlue : "#333336"
                    Text { anchors.centerIn: parent; text: Math.round(modelData * 100) + "%"; color: "#ffffff"; font.pixelSize: 9 }
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (bridge) bridge.setOpacity(modelData) }
                }
            }
        }

        // Measure Tool Context Controls
        RowLayout {
            visible: bridge && bridge.tool === "measure"
            spacing: 8
            RowLayout {
                spacing: 4
                Text { text: Phosphor.ruler; color: contextToolbar.accentCyan; font.family: "Phosphor"; font.pixelSize: 13 }
                Text { text: "Medição:"; color: contextToolbar.accentCyan; font.pixelSize: 11; font.bold: true }
            }
            Text { text: "D: " + Math.round(contextToolbar.measureDistance) + " px"; color: "#fef08a"; font.pixelSize: 10; font.bold: true }
            Text { text: "θ: " + Math.round(contextToolbar.measureAngle) + "°"; color: "#a5f3fc"; font.pixelSize: 10 }
            Text { text: "Arraste no canvas para medir"; color: contextToolbar.textDim; font.pixelSize: 9 }
        }

        // Eyedropper Tool Context Controls
        RowLayout {
            visible: bridge && bridge.tool === "eyedropper"
            spacing: 6
            RowLayout {
                spacing: 4
                Text { text: Phosphor.eyedropper; color: contextToolbar.accentCyan; font.family: "Phosphor"; font.pixelSize: 13 }
                Text { text: "Conta-gotas:"; color: contextToolbar.accentCyan; font.pixelSize: 11; font.bold: true }
            }
            Rectangle {
                width: 16; height: 16; radius: 8
                color: bridge ? bridge.currentFill : "#ffffff"
                border.color: "#ffffff"; border.width: 1
            }
            Text { text: bridge ? bridge.currentFill.toUpperCase() : "#FFFFFF"; color: contextToolbar.textPrimary; font.pixelSize: 10 }
            Text { text: "(Clique para capturar cor)"; color: contextToolbar.textDim; font.pixelSize: 9 }
        }

        // Frame Text Tool Context Controls
        RowLayout {
            visible: bridge && bridge.tool === "text"
            spacing: 6

            RowLayout {
                spacing: 4
                Text { text: Phosphor.textT; color: contextToolbar.accentCyan; font.family: "Phosphor"; font.pixelSize: 13 }
                Text { text: "Texto:"; color: contextToolbar.accentCyan; font.pixelSize: 11; font.bold: true }
            }

            // Font Family ComboBox
            ComboBox {
                id: fontCombo
                Layout.preferredWidth: 100
                height: 22
                model: ["Inter", "Roboto", "Segoe UI", "Arial", "Georgia", "Courier New"]
                currentIndex: Math.max(0, model.indexOf(bridge ? bridge.selectedFontFamily : "Inter"))
                onActivated: if (bridge) bridge.setTextFontFamily(currentText)
                font.pixelSize: 10
            }

            // Font Size SpinBox
            SpinBox {
                id: fontSizeSpin
                Layout.preferredWidth: 70
                height: 22
                from: 6; to: 144; stepSize: 2
                value: Math.round(bridge ? bridge.selectedFontSize : 14)
                onValueModified: if (bridge) bridge.setTextFontSize(value)
                font.pixelSize: 10
            }

            // Bold Toggle Button
            Rectangle {
                width: 22; height: 22; radius: 3
                color: (bridge && bridge.selectedFontBold) ? contextToolbar.accentBlue : "#333336"
                border.color: "#444448"
                Text {
                    anchors.centerIn: parent
                    text: Phosphor.textB
                    color: "#ffffff"
                    font.family: "Phosphor"
                    font.pixelSize: 12
                    font.bold: true
                }
                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: if (bridge) bridge.setTextBold(!bridge.selectedFontBold)
                }
            }

            // Italic Toggle Button
            Rectangle {
                width: 22; height: 22; radius: 3
                color: (bridge && bridge.selectedFontItalic) ? contextToolbar.accentBlue : "#333336"
                border.color: "#444448"
                Text {
                    anchors.centerIn: parent
                    text: Phosphor.textItalic
                    color: "#ffffff"
                    font.family: "Phosphor"
                    font.pixelSize: 12
                }
                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: if (bridge) bridge.setTextItalic(!bridge.selectedFontItalic)
                }
            }

            // Alignment Buttons (Left, Center, Right)
            RowLayout {
                spacing: 2
                Rectangle {
                    width: 22; height: 22; radius: 3
                    color: (bridge && bridge.selectedTextAlign === "left") ? contextToolbar.accentBlue : "#333336"
                    border.color: "#444448"
                    Text { anchors.centerIn: parent; text: Phosphor.textAlignLeft; color: "#ffffff"; font.family: "Phosphor"; font.pixelSize: 12 }
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (bridge) bridge.setTextAlign("left") }
                }
                Rectangle {
                    width: 22; height: 22; radius: 3
                    color: (bridge && bridge.selectedTextAlign === "center") ? contextToolbar.accentBlue : "#333336"
                    border.color: "#444448"
                    Text { anchors.centerIn: parent; text: Phosphor.textAlignCenter; color: "#ffffff"; font.family: "Phosphor"; font.pixelSize: 12 }
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (bridge) bridge.setTextAlign("center") }
                }
                Rectangle {
                    width: 22; height: 22; radius: 3
                    color: (bridge && bridge.selectedTextAlign === "right") ? contextToolbar.accentBlue : "#333336"
                    border.color: "#444448"
                    Text { anchors.centerIn: parent; text: Phosphor.textAlignRight; color: "#ffffff"; font.family: "Phosphor"; font.pixelSize: 12 }
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (bridge) bridge.setTextAlign("right") }
                }
            }

            Rectangle { width: 1; height: 14; color: "#38383a" }

            // Content text quick input
            TextField {
                id: textQuickInput
                Layout.preferredWidth: 140
                height: 22
                text: bridge ? bridge.selectedText : ""
                placeholderText: "Conteúdo do texto..."
                color: contextToolbar.textPrimary
                font.pixelSize: 10
                background: Rectangle { color: "#1c1c1e"; radius: 3; border.color: "#3e3e42" }
                onEditingFinished: {
                    if (bridge && bridge.selectedId) bridge.setProp("text", text)
                }
            }
        }

        // Place Image Tool Context Controls
        RowLayout {
            visible: bridge && bridge.tool === "place"
            spacing: 6
            RowLayout {
                spacing: 4
                Text { text: Phosphor.image; color: contextToolbar.accentCyan; font.family: "Phosphor"; font.pixelSize: 13 }
                Text { text: "Inserir Imagem:"; color: contextToolbar.accentCyan; font.pixelSize: 11; font.bold: true }
            }
            Rectangle {
                height: 22; Layout.preferredWidth: 120; radius: 3; color: contextToolbar.accentBlue
                Text { anchors.centerIn: parent; text: "Carregar do Disco..."; color: "#ffffff"; font.pixelSize: 9; font.bold: true }
                MouseArea {
                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                    onClicked: contextToolbar.requestPlaceImage()
                }
            }
            Rectangle {
                height: 22; Layout.preferredWidth: 95; radius: 3; color: "#333336"; border.color: "#444448"
                Text { anchors.centerIn: parent; text: "Inserir Mockup"; color: contextToolbar.textPrimary; font.pixelSize: 9 }
                MouseArea {
                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                    onClicked: if (bridge) bridge.placeImage("placeholder.png", 200, 200, 400, 260)
                }
            }
        }

        // Vector Crop Tool Context Controls
        RowLayout {
            visible: bridge && bridge.tool === "crop"
            spacing: 6
            RowLayout {
                spacing: 4
                Text { text: Phosphor.crop; color: contextToolbar.accentCyan; font.family: "Phosphor"; font.pixelSize: 13 }
                Text { text: "Recorte Vetorial:"; color: contextToolbar.accentCyan; font.pixelSize: 11; font.bold: true }
            }
            Text { text: "Ajuste a moldura do objeto selecionado"; color: contextToolbar.textDim; font.pixelSize: 9 }
        }

        // Knife (Cut) Tool Context Controls
        RowLayout {
            visible: bridge && bridge.tool === "knife"
            spacing: 8
            RowLayout {
                spacing: 4
                Text { text: Phosphor.knife; color: contextToolbar.accentCyan; font.family: "Phosphor"; font.pixelSize: 13 }
                Text { text: "Faca (Estilete Vetorial):"; color: contextToolbar.accentCyan; font.pixelSize: 11; font.bold: true }
            }
            Text { text: "Arraste uma linha reta sobre as formas selecionadas para fatiá-las"; color: contextToolbar.textDim; font.pixelSize: 10 }
            Rectangle { width: 1; height: 14; color: "#38383a" }
            Rectangle {
                height: 20; Layout.preferredWidth: 70; radius: 3; color: "#252528"; border.color: "#38383a"
                Text { anchors.centerIn: parent; text: "Auto-fechar: Sim"; color: contextToolbar.textSecondary; font.pixelSize: 9 }
            }
        }

        // Pixel Brush Tool Context Controls
        RowLayout {
            visible: bridge && bridge.tool === "pixel_brush"
            spacing: 8
            RowLayout {
                spacing: 4
                Text { text: Phosphor.paintBrush; color: contextToolbar.accentCyan; font.family: "Phosphor"; font.pixelSize: 13 }
                Text { text: "Pincel de Pixel:"; color: contextToolbar.accentCyan; font.pixelSize: 11; font.bold: true }
            }
            Text { text: "Tamanho:"; color: contextToolbar.textSecondary; font.pixelSize: 10 }
            SpinBox {
                from: 1; to: 200; stepSize: 2
                value: Math.round(bridge ? bridge.currentStrokeWidth * 2 : 2)
                onValueModified: if (bridge) bridge.setStrokeWidth(value / 2.0)
            }
            Text { text: "Opacidade:"; color: contextToolbar.textSecondary; font.pixelSize: 10 }
            PetuniaSlider {
                Layout.preferredWidth: 64
                from: 0.0; to: 1.0; stepSize: 0.05
                value: bridge ? bridge.selectedOpacity : 1.0
                onMoved: if (bridge) bridge.setOpacity(value)
            }
        }

        // Pixel Pencil Context Controls
        RowLayout {
            visible: bridge && bridge.tool === "pixel_pencil"
            spacing: 8
            RowLayout {
                spacing: 4
                Text { text: Phosphor.pencilSimple; color: contextToolbar.accentCyan; font.family: "Phosphor"; font.pixelSize: 13 }
                Text { text: "Lápis de Pixel (1px):"; color: contextToolbar.accentCyan; font.pixelSize: 11; font.bold: true }
            }
            Text { text: "Traço duro contínuo de 1 pixel sem interpolação"; color: contextToolbar.textDim; font.pixelSize: 10 }
        }

        // Pixel Eraser Context Controls
        RowLayout {
            visible: bridge && bridge.tool === "pixel_eraser"
            spacing: 8
            RowLayout {
                spacing: 4
                Text { text: Phosphor.eraser; color: contextToolbar.accentCyan; font.family: "Phosphor"; font.pixelSize: 13 }
                Text { text: "Borracha de Pixel:"; color: contextToolbar.accentCyan; font.pixelSize: 11; font.bold: true }
            }
            Text { text: "Tamanho:"; color: contextToolbar.textSecondary; font.pixelSize: 10 }
            SpinBox {
                from: 1; to: 200; stepSize: 2
                value: 20
                onValueModified: {}
            }
            Text { text: "Dureza: 100%"; color: contextToolbar.textSecondary; font.pixelSize: 10 }
        }

        // Pixel Selection (Marquee) Context Controls
        RowLayout {
            visible: bridge && bridge.tool === "pixel_select"
            spacing: 8
            RowLayout {
                spacing: 4
                Text { text: Phosphor.selection; color: contextToolbar.accentCyan; font.family: "Phosphor"; font.pixelSize: 13 }
                Text { text: "Seleção de Pixel (Marquee):"; color: contextToolbar.accentCyan; font.pixelSize: 11; font.bold: true }
            }
            Text { text: "Modo: Nova Seleção"; color: contextToolbar.textSecondary; font.pixelSize: 10 }
            Text { text: "Suavização: 0 px"; color: contextToolbar.textDim; font.pixelSize: 10 }
        }

        // Pixel Lasso Tool Context Controls
        RowLayout {
            visible: bridge && bridge.tool === "pixel_lasso"
            spacing: 8
            RowLayout {
                spacing: 4
                Text { text: Phosphor.lasso; color: contextToolbar.accentCyan; font.family: "Phosphor"; font.pixelSize: 13 }
                Text { text: "Laço Livre (Lasso):"; color: contextToolbar.accentCyan; font.pixelSize: 11; font.bold: true }
            }
            Text { text: "Modo: Nova Seleção"; color: contextToolbar.textSecondary; font.pixelSize: 10 }
            Rectangle { width: 1; height: 14; color: "#38383a" }
            Text { text: "Dica: Segure Shift para adicionar à seleção existente"; color: contextToolbar.textDim; font.pixelSize: 9 }
        }

        // Pixel Clone Stamp Context Controls
        RowLayout {
            visible: bridge && bridge.tool === "pixel_clone"
            spacing: 8
            RowLayout {
                spacing: 4
                Text { text: Phosphor.stamp; color: contextToolbar.accentCyan; font.family: "Phosphor"; font.pixelSize: 13 }
                Text { text: "Carimbo de Clonagem:"; color: contextToolbar.accentCyan; font.pixelSize: 11; font.bold: true }
            }
            Text {
                text: (bridge && bridge.cloneSourceSet)
                    ? ("Origem: (" + Math.round(bridge.cloneSourceX) + ", " + Math.round(bridge.cloneSourceY) + ")")
                    : "Alt+Clique no canvas para definir origem"
                color: (bridge && bridge.cloneSourceSet) ? contextToolbar.accentCyan : "#f59e0b"
                font.pixelSize: 10
                font.bold: bridge && bridge.cloneSourceSet
            }
            Rectangle {
                height: 20; Layout.preferredWidth: 84; radius: 3; color: "#333336"; border.color: "#444448"
                visible: bridge && bridge.cloneSourceSet
                Text { anchors.centerIn: parent; text: "Limpar Origem"; color: contextToolbar.textPrimary; font.pixelSize: 9 }
                MouseArea {
                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                    onClicked: if (bridge) bridge.clearCloneSource()
                }
            }
            Rectangle { width: 1; height: 14; color: "#38383a" }
            Text { text: "Tamanho:"; color: contextToolbar.textSecondary; font.pixelSize: 10 }
            SpinBox {
                from: 2; to: 200; stepSize: 4
                value: Math.round(bridge ? bridge.currentStrokeWidth * 2 : 24)
                onValueModified: if (bridge) bridge.setStrokeWidth(value / 2.0)
            }
        }

        // Pixel Dodge / Burn Context Controls
        RowLayout {
            visible: bridge && (bridge.tool === "pixel_dodge" || bridge.tool === "pixel_burn")
            spacing: 8
            RowLayout {
                spacing: 4
                Text {
                    text: (bridge && bridge.tool === "pixel_burn") ? Phosphor.moon : Phosphor.sun
                    color: contextToolbar.accentCyan
                    font.family: "Phosphor"
                    font.pixelSize: 13
                }
                Text {
                    text: (bridge && bridge.tool === "pixel_burn") ? "Subexposição (Burn):" : "Superexposição (Dodge):"
                    color: contextToolbar.accentCyan
                    font.pixelSize: 11
                    font.bold: true
                }
            }
            Text { text: "Alcance: Médios"; color: contextToolbar.textSecondary; font.pixelSize: 10 }
            Rectangle { width: 1; height: 14; color: "#38383a" }
            Text { text: "Exposição:"; color: contextToolbar.textSecondary; font.pixelSize: 10 }
            Repeater {
                model: [0.25, 0.50, 0.75]
                delegate: Rectangle {
                    height: 20; Layout.preferredWidth: 32; radius: 3
                    color: (bridge && Math.abs(bridge.selectedOpacity - modelData) < 0.1) ? contextToolbar.accentBlue : "#333336"
                    Text { anchors.centerIn: parent; text: Math.round(modelData * 100) + "%"; color: "#ffffff"; font.pixelSize: 8 }
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (bridge) bridge.setOpacity(modelData) }
                }
            }
            Rectangle { width: 1; height: 14; color: "#38383a" }
            Text { text: "Tamanho:"; color: contextToolbar.textSecondary; font.pixelSize: 10 }
            SpinBox {
                from: 4; to: 200; stepSize: 4
                value: Math.round(bridge ? bridge.currentStrokeWidth * 2 : 30)
                onValueModified: if (bridge) bridge.setStrokeWidth(value / 2.0)
            }
        }

        // Pixel Smudge / Blur Context Controls
        RowLayout {
            visible: bridge && (bridge.tool === "pixel_smudge" || bridge.tool === "pixel_blur")
            spacing: 8
            RowLayout {
                spacing: 4
                Text {
                    text: (bridge && bridge.tool === "pixel_blur") ? Phosphor.drop : Phosphor.handPointing
                    color: contextToolbar.accentCyan
                    font.family: "Phosphor"
                    font.pixelSize: 13
                }
                Text {
                    text: (bridge && bridge.tool === "pixel_blur") ? "Desfoque (Blur):" : "Borrar (Smudge):"
                    color: contextToolbar.accentCyan
                    font.pixelSize: 11
                    font.bold: true
                }
            }
            Text { text: "Intensidade:"; color: contextToolbar.textSecondary; font.pixelSize: 10 }
            Repeater {
                model: [0.25, 0.50, 0.75, 1.0]
                delegate: Rectangle {
                    height: 20; Layout.preferredWidth: 32; radius: 3
                    color: (bridge && Math.abs(bridge.selectedOpacity - modelData) < 0.1) ? contextToolbar.accentBlue : "#333336"
                    Text { anchors.centerIn: parent; text: Math.round(modelData * 100) + "%"; color: "#ffffff"; font.pixelSize: 8 }
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (bridge) bridge.setOpacity(modelData) }
                }
            }
            Rectangle { width: 1; height: 14; color: "#38383a" }
            Text { text: "Tamanho:"; color: contextToolbar.textSecondary; font.pixelSize: 10 }
            SpinBox {
                from: 4; to: 200; stepSize: 4
                value: Math.round(bridge ? bridge.currentStrokeWidth * 2 : 24)
                onValueModified: if (bridge) bridge.setStrokeWidth(value / 2.0)
            }
        }

        // Flood Fill Context Controls
        RowLayout {
            visible: bridge && bridge.tool === "pixel_fill"
            spacing: 8
            RowLayout {
                spacing: 4
                Text { text: Phosphor.paintBucket; color: contextToolbar.accentCyan; font.family: "Phosphor"; font.pixelSize: 13 }
                Text { text: "Balde de Tinta:"; color: contextToolbar.accentCyan; font.pixelSize: 11; font.bold: true }
            }
            Text { text: "Tolerância: 20%"; color: contextToolbar.textSecondary; font.pixelSize: 10 }
            Text { text: "Contíguo: Sim"; color: contextToolbar.textDim; font.pixelSize: 10 }
        }

        // Slice Tool Context Controls
        RowLayout {
            visible: bridge && (bridge.tool === "slice" || bridge.tool === "slice_select")
            spacing: 8
            RowLayout {
                spacing: 4
                Text { text: Phosphor.knife; color: contextToolbar.accentCyan; font.family: "Phosphor"; font.pixelSize: 13 }
                Text { text: "Ferramenta de Fatia (Slice):"; color: contextToolbar.accentCyan; font.pixelSize: 11; font.bold: true }
            }
            Text { text: "Exportar seleção como fatia independente"; color: contextToolbar.textDim; font.pixelSize: 10 }
            Rectangle {
                height: 22; Layout.preferredWidth: 92; radius: 4; color: contextToolbar.accentBlue
                RowLayout {
                    anchors.centerIn: parent; spacing: 4
                    Text { text: Phosphor.fileArrowDown; color: "#ffffff"; font.family: "Phosphor"; font.pixelSize: 12 }
                    Text { text: "Exportar"; color: "#ffffff"; font.pixelSize: 10; font.bold: true }
                }
                MouseArea {
                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                    onClicked: if (bridge) bridge.exportPng("/tmp/petunia-design-export.png")
                }
            }
        }

        Item { Layout.fillWidth: true }

        // System Message
        Text {
            text: bridge ? bridge.message : ""
            color: "#38bdf8"
            font.pixelSize: 10
            Layout.alignment: Qt.AlignVCenter
        }
    }
}
