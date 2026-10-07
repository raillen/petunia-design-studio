import QtQuick
import QtQuick.Controls

Slider {
    id: control
    implicitHeight: 20

    property color trackColor: "#1e1e20"
    property color progressColor: "#00b4d8"
    property color handleColor: control.pressed ? "#00b4d8" : (control.hovered ? "#ffffff" : "#d1d1d6")

    background: Rectangle {
        x: control.leftPadding
        y: control.topPadding + control.availableHeight / 2 - height / 2
        implicitWidth: 120
        implicitHeight: 4
        width: control.availableWidth
        height: implicitHeight
        radius: 2
        color: control.trackColor
        border.color: "#38383c"
        border.width: 1

        Rectangle {
            width: control.visualPosition * parent.width
            height: parent.height
            color: control.progressColor
            radius: 2
        }
    }

    handle: Rectangle {
        x: control.leftPadding + control.visualPosition * (control.availableWidth - width)
        y: control.topPadding + control.availableHeight / 2 - height / 2
        implicitWidth: 12
        implicitHeight: 12
        radius: 6
        color: control.handleColor
        border.color: "#181819"
        border.width: 1
    }
}
