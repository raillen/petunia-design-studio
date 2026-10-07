import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: topHeaderBar
    width: parent ? parent.width : 1280
    height: 38

    property color bgHeader: "#252526"
    property color borderDark: "#181819"
    property color accentCyan: "#00b4d8"
    property color accentBlue: "#1976d2"
    property color textPrimary: "#f2f2f5"
    property color textSecondary: "#a1a1a6"
    property color textDim: "#6c6c70"

    signal requestHelp()
    signal requestStylesTab()
    signal requestExport()

    color: bgHeader
    border.color: borderDark
    border.width: 1

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: 8
        anchors.rightMargin: 10
        spacing: 8

        // App Logo (Stylized 'A' mark)
        Rectangle {
            width: 26
            height: 26
            radius: 5
            color: "#2f2f32"
            border.color: "#404044"
            Text {
                anchors.centerIn: parent
                text: "A"
                color: "#ffffff"
                font.pixelSize: 15
                font.bold: true
                font.family: "Inter, Segoe UI, sans-serif"
            }
        }

        // Persona Switcher Pills (Vector, Pixel, Export)
        Rectangle {
            height: 28
            Layout.preferredWidth: 220
            radius: 14
            color: "#181819"
            border.color: "#2d2d30"

            RowLayout {
                anchors.fill: parent
                spacing: 2
                anchors.margins: 2

                // Vector Persona
                Rectangle {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    radius: 12
                    color: (bridge && bridge.persona === "vector") ? topHeaderBar.accentCyan : "transparent"
                    RowLayout {
                        anchors.centerIn: parent
                        spacing: 4
                        Text {
                            text: Phosphor.penNib
                            color: (bridge && bridge.persona === "vector") ? "#ffffff" : topHeaderBar.textSecondary
                            font.family: "Phosphor"
                            font.pixelSize: 13
                        }
                        Text {
                            text: "Vector"
                            color: (bridge && bridge.persona === "vector") ? "#ffffff" : topHeaderBar.textSecondary
                            font.pixelSize: 11
                            font.bold: bridge && bridge.persona === "vector"
                        }
                    }
                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            if (bridge) {
                                bridge.setPersona("vector")
                                bridge.setTool("select")
                            }
                        }
                    }
                }

                // Pixel Persona
                Rectangle {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    radius: 12
                    color: (bridge && bridge.persona === "pixel") ? topHeaderBar.accentCyan : "transparent"
                    RowLayout {
                        anchors.centerIn: parent
                        spacing: 4
                        Text {
                            text: Phosphor.gridFour
                            color: (bridge && bridge.persona === "pixel") ? "#ffffff" : topHeaderBar.textDim
                            font.family: "Phosphor"
                            font.pixelSize: 13
                        }
                        Text {
                            text: "Pixel"
                            color: (bridge && bridge.persona === "pixel") ? "#ffffff" : topHeaderBar.textDim
                            font.pixelSize: 11
                            font.bold: bridge && bridge.persona === "pixel"
                        }
                    }
                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            if (bridge) {
                                bridge.setPersona("pixel")
                                bridge.setTool("pixel_brush")
                            }
                        }
                    }
                }

                // Export Persona
                Rectangle {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    radius: 12
                    color: (bridge && bridge.persona === "export") ? topHeaderBar.accentCyan : "transparent"
                    RowLayout {
                        anchors.centerIn: parent
                        spacing: 4
                        Text {
                            text: Phosphor.exportIcon
                            color: (bridge && bridge.persona === "export") ? "#ffffff" : topHeaderBar.textDim
                            font.family: "Phosphor"
                            font.pixelSize: 13
                        }
                        Text {
                            text: "Export"
                            color: (bridge && bridge.persona === "export") ? "#ffffff" : topHeaderBar.textDim
                            font.pixelSize: 11
                            font.bold: bridge && bridge.persona === "export"
                        }
                    }
                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            if (bridge) {
                                bridge.setPersona("export")
                                bridge.setTool("slice")
                            }
                        }
                    }
                }
            }
        }

        // Typography & Assets pill buttons
        Rectangle {
            height: 26
            Layout.preferredWidth: 64
            radius: 5
            color: "#28282b"
            border.color: "#3a3a3d"
            RowLayout {
                anchors.fill: parent
                spacing: 0
                Rectangle {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    color: "transparent"
                    Text {
                        anchors.centerIn: parent
                        text: Phosphor.textT
                        color: topHeaderBar.textSecondary
                        font.family: "Phosphor"
                        font.pixelSize: 14
                    }
                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: topHeaderBar.requestStylesTab()
                    }
                }
                Rectangle { width: 1; height: 16; color: "#38383a" }
                Rectangle {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    color: "transparent"
                    Text {
                        anchors.centerIn: parent
                        text: Phosphor.shapes
                        color: topHeaderBar.textSecondary
                        font.family: "Phosphor"
                        font.pixelSize: 14
                    }
                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: topHeaderBar.requestStylesTab()
                    }
                }
            }
        }

        Item { Layout.fillWidth: true }

        // Magnet / Snapping Toggle
        Rectangle {
            width: 28
            height: 26
            radius: 4
            color: (bridge && bridge.snappingEnabled) ? "#2a3a48" : "transparent"
            border.color: (bridge && bridge.snappingEnabled) ? topHeaderBar.accentCyan : "transparent"
            Text {
                anchors.centerIn: parent
                text: Phosphor.magnet
                color: (bridge && bridge.snappingEnabled) ? topHeaderBar.accentCyan : topHeaderBar.textSecondary
                font.family: "Phosphor"
                font.pixelSize: 16
                opacity: (bridge && bridge.snappingEnabled) ? 1.0 : 0.6
            }
            MouseArea {
                id: snapMouse
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: if (bridge) bridge.toggleSnapping()
            }
            ToolTip.visible: snapMouse.containsMouse
            ToolTip.text: "Snap " + ((bridge && bridge.snappingEnabled) ? "Ativado (8px)" : "Desativado")
        }

        // Alignment Toolbar Group
        RowLayout {
            spacing: 2
            Rectangle {
                width: 24; height: 24; radius: 3; color: "transparent"
                Text { anchors.centerIn: parent; text: Phosphor.alignLeft; color: topHeaderBar.textSecondary; font.family: "Phosphor"; font.pixelSize: 15 }
                MouseArea { id: maAl; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: if (bridge) bridge.alignSelected("left") }
                ToolTip.visible: maAl.containsMouse; ToolTip.text: "Alinhar à Esquerda"
            }
            Rectangle {
                width: 24; height: 24; radius: 3; color: "transparent"
                Text { anchors.centerIn: parent; text: Phosphor.alignCenterHorizontal; color: topHeaderBar.textSecondary; font.family: "Phosphor"; font.pixelSize: 15 }
                MouseArea { id: maAc; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: if (bridge) bridge.alignSelected("center") }
                ToolTip.visible: maAc.containsMouse; ToolTip.text: "Alinhar ao Centro H"
            }
            Rectangle {
                width: 24; height: 24; radius: 3; color: "transparent"
                Text { anchors.centerIn: parent; text: Phosphor.alignRight; color: topHeaderBar.textSecondary; font.family: "Phosphor"; font.pixelSize: 15 }
                MouseArea { id: maAr; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: if (bridge) bridge.alignSelected("right") }
                ToolTip.visible: maAr.containsMouse; ToolTip.text: "Alinhar à Direita"
            }
            Rectangle {
                width: 24; height: 24; radius: 3; color: "transparent"
                Text { anchors.centerIn: parent; text: Phosphor.alignTop; color: topHeaderBar.textSecondary; font.family: "Phosphor"; font.pixelSize: 15 }
                MouseArea { id: maAt; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: if (bridge) bridge.alignSelected("top") }
                ToolTip.visible: maAt.containsMouse; ToolTip.text: "Alinhar ao Topo"
            }
            Rectangle {
                width: 24; height: 24; radius: 3; color: "transparent"
                Text { anchors.centerIn: parent; text: Phosphor.alignCenterVertical; color: topHeaderBar.textSecondary; font.family: "Phosphor"; font.pixelSize: 15 }
                MouseArea { id: maAm; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: if (bridge) bridge.alignSelected("middle") }
                ToolTip.visible: maAm.containsMouse; ToolTip.text: "Alinhar ao Meio V"
            }
            Rectangle {
                width: 24; height: 24; radius: 3; color: "transparent"
                Text { anchors.centerIn: parent; text: Phosphor.alignBottom; color: topHeaderBar.textSecondary; font.family: "Phosphor"; font.pixelSize: 15 }
                MouseArea { id: maAb; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: if (bridge) bridge.alignSelected("bottom") }
                ToolTip.visible: maAb.containsMouse; ToolTip.text: "Alinhar à Base"
            }
        }

        Rectangle { width: 1; height: 18; color: "#38383a" }

        // Boolean Operations Toolbar Group (Exact Affinity Geometry Icons)
        RowLayout {
            spacing: 2
            Rectangle {
                width: 24; height: 24; radius: 3; color: "transparent"
                Text { anchors.centerIn: parent; text: Phosphor.unite; color: (bridge && bridge.selectedIds && bridge.selectedIds.length > 1) ? topHeaderBar.textPrimary : topHeaderBar.textDim; font.family: "Phosphor"; font.pixelSize: 15 }
                MouseArea {
                    id: maBu
                    anchors.fill: parent
                    enabled: bridge && bridge.selectedIds && bridge.selectedIds.length > 1
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: if (bridge) bridge.boolean("union")
                }
                ToolTip.visible: maBu.containsMouse; ToolTip.text: "Unir (Add)"
            }
            Rectangle {
                width: 24; height: 24; radius: 3; color: "transparent"
                Text { anchors.centerIn: parent; text: Phosphor.subtract; color: (bridge && bridge.selectedIds && bridge.selectedIds.length > 1) ? topHeaderBar.textPrimary : topHeaderBar.textDim; font.family: "Phosphor"; font.pixelSize: 15 }
                MouseArea {
                    id: maBs
                    anchors.fill: parent
                    enabled: bridge && bridge.selectedIds && bridge.selectedIds.length > 1
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: if (bridge) bridge.boolean("subtract")
                }
                ToolTip.visible: maBs.containsMouse; ToolTip.text: "Subtrair (Subtract)"
            }
            Rectangle {
                width: 24; height: 24; radius: 3; color: "transparent"
                Text { anchors.centerIn: parent; text: Phosphor.intersect; color: (bridge && bridge.selectedIds && bridge.selectedIds.length > 1) ? topHeaderBar.textPrimary : topHeaderBar.textDim; font.family: "Phosphor"; font.pixelSize: 15 }
                MouseArea {
                    id: maBi
                    anchors.fill: parent
                    enabled: bridge && bridge.selectedIds && bridge.selectedIds.length > 1
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: if (bridge) bridge.boolean("intersect")
                }
                ToolTip.visible: maBi.containsMouse; ToolTip.text: "Intersectar (Intersect)"
            }
            Rectangle {
                width: 24; height: 24; radius: 3; color: "transparent"
                Text { anchors.centerIn: parent; text: Phosphor.exclude; color: (bridge && bridge.selectedIds && bridge.selectedIds.length > 1) ? topHeaderBar.textPrimary : topHeaderBar.textDim; font.family: "Phosphor"; font.pixelSize: 15 }
                MouseArea {
                    id: maBx
                    anchors.fill: parent
                    enabled: bridge && bridge.selectedIds && bridge.selectedIds.length > 1
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: if (bridge) bridge.boolean("xor")
                }
                ToolTip.visible: maBx.containsMouse; ToolTip.text: "XOR (Excluir sobreposição)"
            }
            Rectangle {
                width: 24; height: 24; radius: 3; color: "transparent"
                Text { anchors.centerIn: parent; text: Phosphor.divide; color: (bridge && bridge.selectedIds && bridge.selectedIds.length > 1) ? topHeaderBar.textPrimary : topHeaderBar.textDim; font.family: "Phosphor"; font.pixelSize: 15 }
                MouseArea {
                    id: maBd
                    anchors.fill: parent
                    enabled: bridge && bridge.selectedIds && bridge.selectedIds.length > 1
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: if (bridge) bridge.boolean("divide")
                }
                ToolTip.visible: maBd.containsMouse; ToolTip.text: "Dividir (Divide)"
            }
        }

        Rectangle { width: 1; height: 18; color: "#38383a" }

        // Insert Modes (Behind, Inside, Top)
        RowLayout {
            spacing: 2
            Rectangle {
                width: 22; height: 22; radius: 3; color: "transparent"
                Text { anchors.centerIn: parent; text: Phosphor.selectionBackground; color: topHeaderBar.textSecondary; font.family: "Phosphor"; font.pixelSize: 14 }
                MouseArea { id: maIb; anchors.fill: parent; hoverEnabled: true }
                ToolTip.visible: maIb.containsMouse; ToolTip.text: "Inserir Atrás"
            }
            Rectangle {
                width: 22; height: 22; radius: 3; color: "transparent"
                Text { anchors.centerIn: parent; text: Phosphor.selection; color: topHeaderBar.textSecondary; font.family: "Phosphor"; font.pixelSize: 14 }
                MouseArea { id: maIi; anchors.fill: parent; hoverEnabled: true }
                ToolTip.visible: maIi.containsMouse; ToolTip.text: "Inserir Dentro"
            }
            Rectangle {
                width: 22; height: 22; radius: 3; color: "transparent"
                Text { anchors.centerIn: parent; text: Phosphor.selectionForeground; color: topHeaderBar.textSecondary; font.family: "Phosphor"; font.pixelSize: 14 }
                MouseArea { id: maIt; anchors.fill: parent; hoverEnabled: true }
                ToolTip.visible: maIt.containsMouse; ToolTip.text: "Inserir no Topo"
            }
        }

        // Help Button
        Rectangle {
            width: 22
            height: 22
            radius: 11
            color: "#2f2f32"
            border.color: "#404044"
            Text {
                anchors.centerIn: parent
                text: Phosphor.question
                color: topHeaderBar.textSecondary
                font.family: "Phosphor"
                font.pixelSize: 13
                font.bold: true
            }
            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: topHeaderBar.requestHelp()
            }
        }

        // Export PNG Button
        Rectangle {
            height: 26
            Layout.preferredWidth: 96
            radius: 5
            color: "#2d2d30"
            border.color: "#48484c"
            RowLayout {
                anchors.centerIn: parent
                spacing: 4
                Text {
                    text: Phosphor.fileArrowDown
                    color: "#ffffff"
                    font.family: "Phosphor"
                    font.pixelSize: 13
                }
                Text {
                    text: "Export PNG"
                    color: "#ffffff"
                    font.pixelSize: 11
                    font.bold: true
                }
            }
            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: {
                    if (topHeaderBar.requestExport.length !== undefined) {
                        topHeaderBar.requestExport()
                    }
                    bridge.exportPng("/tmp/petunia-design-export.png")
                }
            }
        }
    }
}
