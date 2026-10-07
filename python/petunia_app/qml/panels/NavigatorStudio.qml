import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components"

Item {
    id: navigatorStudio
    Layout.fillWidth: true
    Layout.fillHeight: true

    property color textPrimary: "#f2f2f5"
    property color textSecondary: "#a1a1a6"
    property color textDim: "#6c6c70"
    property color bgField: "#181819"
    property color borderSubtle: "#333335"
    property color accentCyan: "#00b4d8"

    property real panX: 0
    property real panY: 0
    property real canvasWidth: 800
    property real canvasHeight: 600
    property real totalDocWidth: 6000
    property real totalDocHeight: 4000

    signal panRequested(real newPanX, real newPanY)

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 6
        spacing: 4

        // Zoom Slider with - and + and %
        RowLayout {
            Layout.fillWidth: true
            spacing: 4

            Text {
                text: Phosphor.minus
                color: navigatorStudio.textSecondary
                font.family: "Phosphor"
                font.pixelSize: 11
                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: bridge.zoomOut()
                }
            }

            PetuniaSlider {
                Layout.fillWidth: true
                from: 0.1
                to: 3.0
                value: bridge.zoom
                onMoved: bridge.setZoom(value)
            }

            Text {
                text: Phosphor.plus
                color: navigatorStudio.textSecondary
                font.family: "Phosphor"
                font.pixelSize: 11
                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: bridge.zoomIn()
                }
            }

            Text {
                text: Math.round(bridge.zoom * 100) + " %"
                color: navigatorStudio.textPrimary
                font.pixelSize: 10
                font.bold: true
                Layout.preferredWidth: 36
                horizontalAlignment: Text.AlignRight
            }
        }

        // View Point Dropdown
        Rectangle {
            Layout.fillWidth: true
            height: 20
            color: navigatorStudio.bgField
            border.color: navigatorStudio.borderSubtle
            radius: 3

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 6
                anchors.rightMargin: 6
                Text { text: "View Point 1"; color: navigatorStudio.textSecondary; font.pixelSize: 10; Layout.fillWidth: true }
                Text { text: Phosphor.caretDown; color: navigatorStudio.textDim; font.family: "Phosphor"; font.pixelSize: 9 }
            }
        }

        // Navigator Minimap Thumbnail (Exact Affinity Designer Navigator Box!)
        Rectangle {
            id: minimapBox
            Layout.preferredWidth: 160
            Layout.preferredHeight: 96
            Layout.alignment: Qt.AlignHCenter
            color: "#181819"
            border.color: "#38383a"
            border.width: 1
            clip: true

            property real totalW: Math.max(200, navigatorStudio.totalDocWidth)
            property real totalH: Math.max(150, navigatorStudio.totalDocHeight)
            property real mapScale: Math.min((width - 16) / totalW, (height - 16) / totalH)

            Item {
                id: minimapContent
                anchors.centerIn: parent
                width: minimapBox.totalW * minimapBox.mapScale
                height: minimapBox.totalH * minimapBox.mapScale

                // Mini Artboard Cards
                Repeater {
                    model: bridge.artboards
                    delegate: Rectangle {
                        x: modelData.x * minimapBox.mapScale
                        y: modelData.y * minimapBox.mapScale
                        width: Math.max(4, modelData.width * minimapBox.mapScale)
                        height: Math.max(4, modelData.height * minimapBox.mapScale)
                        color: modelData.fill || "#ffffff"
                        border.color: modelData.selected ? navigatorStudio.accentCyan : "#48484c"
                        border.width: 1
                    }
                }

                // Red Viewport Rectangle Bounding Active View
                Rectangle {
                    id: navFrustum
                    x: Math.max(0, -navigatorStudio.panX / bridge.zoom * minimapBox.mapScale)
                    y: Math.max(0, -navigatorStudio.panY / bridge.zoom * minimapBox.mapScale)
                    width: Math.max(12, Math.min(parent.width, (navigatorStudio.canvasWidth / bridge.zoom) * minimapBox.mapScale))
                    height: Math.max(8, Math.min(parent.height, (navigatorStudio.canvasHeight / bridge.zoom) * minimapBox.mapScale))
                    color: "transparent"
                    border.color: "#ef4444"
                    border.width: 1.5

                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.SizeAllCursor
                        property real dragStartX: 0
                        property real dragStartY: 0
                        property real startPanX: 0
                        property real startPanY: 0
                        onPressed: function(mouse) {
                            dragStartX = mouse.x
                            dragStartY = mouse.y
                            startPanX = navigatorStudio.panX
                            startPanY = navigatorStudio.panY
                        }
                        onPositionChanged: function(mouse) {
                            if (!pressed) return
                            var dx = (mouse.x - dragStartX) / minimapBox.mapScale * bridge.zoom
                            var dy = (mouse.y - dragStartY) / minimapBox.mapScale * bridge.zoom
                            navigatorStudio.panRequested(startPanX - dx, startPanY - dy)
                        }
                    }
                }
            }

            MouseArea {
                anchors.fill: parent
                z: -1
                onClicked: function(mouse) {
                    var targetDocX = (mouse.x - (minimapBox.width - minimapContent.width) / 2) / minimapBox.mapScale
                    var targetDocY = (mouse.y - (minimapBox.height - minimapContent.height) / 2) / minimapBox.mapScale
                    var newPx = (navigatorStudio.canvasWidth / 2) - targetDocX * bridge.zoom
                    var newPy = (navigatorStudio.canvasHeight / 2) - targetDocY * bridge.zoom
                    navigatorStudio.panRequested(newPx, newPy)
                }
            }
        }
    }
}
