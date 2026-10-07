import QtQuick

Text {
    id: pIcon
    property alias name: pIcon.text
    property alias icon: pIcon.text
    property int size: 14

    color: "#a1a1a6"
    font.family: "Phosphor"
    font.pixelSize: size
    horizontalAlignment: Text.AlignHCenter
    verticalAlignment: Text.AlignVCenter
}
