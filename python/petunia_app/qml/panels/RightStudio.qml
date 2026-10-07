import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: rightStudio
    width: 288
    Layout.preferredWidth: 288
    Layout.fillHeight: true

    property color bgPanel: "#242426"
    property color bgPanelHeader: "#202021"
    property color bgPanelTabActive: "#2b2b2d"
    property color bgPanelTabInactive: "#1f1f20"
    property color bgPanelBody: "#28282a"
    property color borderDark: "#181819"
    property color textSecondary: "#a1a1a6"

    property alias topTab: colourStudioComp.activeTab
    property alias midTab: layersStudioComp.activeTab
    property int bottomTab: 1

    property alias editingStroke: colourStudioComp.editingStroke

    property real panX: 0
    property real panY: 0
    property real canvasWidth: 800
    property real canvasHeight: 600
    property real totalDocWidth: 6000
    property real totalDocHeight: 4000

    signal panRequested(real newPanX, real newPanY)

    color: bgPanel
    border.color: borderDark
    border.width: 1

    ColumnLayout {
        anchors.fill: parent
        spacing: 1

        // -----------------------------------------------------
        // DOCK 1 (TOP): Colour / Swatches / Stroke / Appearance
        // -----------------------------------------------------
        ColourStudio {
            id: colourStudioComp
            Layout.fillWidth: true
        }

        // -----------------------------------------------------
        // DOCK 2 (MIDDLE): Layers / Path Brushes / Quick FX / Styles
        // -----------------------------------------------------
        LayersStudio {
            id: layersStudioComp
            Layout.fillWidth: true
            Layout.fillHeight: true
        }

        // -----------------------------------------------------
        // DOCK 3 (BOTTOM): Transform / Navigator / History
        // -----------------------------------------------------
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 180
            color: rightStudio.bgPanelBody

            ColumnLayout {
                anchors.fill: parent
                spacing: 0

                // Tab Bar
                Rectangle {
                    Layout.fillWidth: true
                    height: 25
                    color: rightStudio.bgPanelHeader

                    RowLayout {
                        anchors.fill: parent
                        spacing: 0
                        Repeater {
                            model: ["Transform", "Navigator", "History"]
                            delegate: Rectangle {
                                Layout.fillWidth: true
                                Layout.fillHeight: true
                                color: rightStudio.bottomTab === index ? rightStudio.bgPanelTabActive : rightStudio.bgPanelTabInactive
                                Text {
                                    anchors.centerIn: parent
                                    text: modelData
                                    color: rightStudio.bottomTab === index ? "#ffffff" : rightStudio.textSecondary
                                    font.pixelSize: 10
                                    font.bold: rightStudio.bottomTab === index
                                }
                                MouseArea {
                                    anchors.fill: parent
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: rightStudio.bottomTab = index
                                }
                            }
                        }
                    }
                }

                // Tab 0: Transform
                TransformStudio {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    visible: rightStudio.bottomTab === 0
                }

                // Tab 1: Navigator
                NavigatorStudio {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    visible: rightStudio.bottomTab === 1
                    panX: rightStudio.panX
                    panY: rightStudio.panY
                    canvasWidth: rightStudio.canvasWidth
                    canvasHeight: rightStudio.canvasHeight
                    totalDocWidth: rightStudio.totalDocWidth
                    totalDocHeight: rightStudio.totalDocHeight
                    onPanRequested: function(nx, ny) {
                        rightStudio.panRequested(nx, ny)
                    }
                }

                // Tab 2: History
                HistoryStudio {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    visible: rightStudio.bottomTab === 2
                }
            }
        }
    }
}
