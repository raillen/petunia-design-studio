import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components"

Item {
    id: historyStudio
    Layout.fillWidth: true
    Layout.fillHeight: true

    property color bgPanel: "#242426"
    property color bgHeader: "#1f1f21"
    property color bgItemHover: "#2d2d30"
    property color bgItemActive: "#323236"
    property color borderSubtle: "#333336"
    property color textPrimary: "#f2f2f5"
    property color textSecondary: "#a1a1a6"
    property color textDim: "#6c6c70"
    property color accentCyan: "#00b4d8"

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        // Sub-header Toolbar
        Rectangle {
            Layout.fillWidth: true
            height: 26
            color: historyStudio.bgHeader
            border.color: historyStudio.borderSubtle
            border.width: 1

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 8
                anchors.rightMargin: 8
                spacing: 6

                PIcon {
                    name: Phosphor.clockCounterClockwise
                    size: 13
                    color: historyStudio.textSecondary
                }

                Text {
                    text: "Histórico"
                    color: historyStudio.textSecondary
                    font.pixelSize: 10
                    font.bold: true
                }

                Text {
                    text: "(" + bridge.historyItems.length + ")"
                    color: historyStudio.textDim
                    font.pixelSize: 9
                }

                Item { Layout.fillWidth: true }

                // Undo Button
                Rectangle {
                    width: 20
                    height: 20
                    radius: 3
                    color: undoMouse.containsMouse && bridge.canUndo ? "#38383c" : "transparent"

                    PIcon {
                        anchors.centerIn: parent
                        name: Phosphor.arrowCounterClockwise
                        size: 12
                        color: bridge.canUndo ? historyStudio.textPrimary : historyStudio.textDim
                    }

                    MouseArea {
                        id: undoMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: bridge.canUndo ? Qt.PointingHandCursor : Qt.ArrowCursor
                        onClicked: if (bridge.canUndo) bridge.undo()
                    }
                }

                // Redo Button
                Rectangle {
                    width: 20
                    height: 20
                    radius: 3
                    color: redoMouse.containsMouse && bridge.canRedo ? "#38383c" : "transparent"

                    PIcon {
                        anchors.centerIn: parent
                        name: Phosphor.arrowClockwise
                        size: 12
                        color: bridge.canRedo ? historyStudio.textPrimary : historyStudio.textDim
                    }

                    MouseArea {
                        id: redoMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: bridge.canRedo ? Qt.PointingHandCursor : Qt.ArrowCursor
                        onClicked: if (bridge.canRedo) bridge.redo()
                    }
                }
            }
        }

        // List or Empty State
        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true

            // Empty State
            ColumnLayout {
                anchors.centerIn: parent
                spacing: 6
                visible: bridge.historyItems.length === 0

                PIcon {
                    Layout.alignment: Qt.AlignHCenter
                    name: Phosphor.clockCounterClockwise
                    size: 24
                    color: historyStudio.textDim
                }

                Text {
                    Layout.alignment: Qt.AlignHCenter
                    text: "Nenhuma ação registrada"
                    color: historyStudio.textDim
                    font.pixelSize: 10
                }
            }

            // History List
            ListView {
                id: historyList
                anchors.fill: parent
                anchors.margins: 4
                model: bridge.historyItems
                clip: true
                spacing: 1
                visible: bridge.historyItems.length > 0

                delegate: Rectangle {
                    id: itemDelegate
                    width: ListView.view.width
                    height: 22
                    radius: 2
                    color: itemMouse.containsMouse ? historyStudio.bgItemHover : (index === bridge.historyItems.length - 1 ? historyStudio.bgItemActive : "transparent")

                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 6
                        anchors.rightMargin: 6
                        spacing: 6

                        Text {
                            text: (index + 1).toString()
                            color: historyStudio.textDim
                            font.pixelSize: 9
                            Layout.preferredWidth: 16
                        }

                        PIcon {
                            name: Phosphor.check
                            size: 10
                            color: index === bridge.historyItems.length - 1 ? historyStudio.accentCyan : historyStudio.textDim
                        }

                        Text {
                            Layout.fillWidth: true
                            text: modelData
                            color: index === bridge.historyItems.length - 1 ? historyStudio.textPrimary : historyStudio.textSecondary
                            font.pixelSize: 10
                            font.bold: index === bridge.historyItems.length - 1
                            elide: Text.ElideRight
                        }
                    }

                    MouseArea {
                        id: itemMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: bridge.jumpHistory(index)
                    }
                }
            }
        }
    }
}
