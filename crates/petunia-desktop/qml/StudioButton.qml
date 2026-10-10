pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls

Button {
    id: control
    required property StudioTheme theme
    property var backend
    property string iconKey: ""
    property string shortcut: ""
    property string explanation: ""
    property bool iconOnly: false
    property bool selected: false
    property bool selectable: false
    checkable: selectable
    checked: selected
    Accessible.checkable: selectable
    Accessible.checked: selected
    property string asset: {
        if (!backend || !iconKey) return ""
        var snapshot = backend.stateJson
        return backend.iconSource(iconKey)
    }
    implicitHeight: theme.target
    implicitWidth: iconOnly ? theme.target : Math.max(theme.target, contentItem.implicitWidth + 20)
    font.pixelSize: 14*theme.uiScale
    hoverEnabled: true
    focusPolicy: Qt.StrongFocus
    Accessible.name: text
    Accessible.description: explanation || (shortcut ? "Atalho: " + shortcut : "")
    ToolTip.text: text + (shortcut ? " · " + shortcut : "") + (explanation ? "\n" + explanation : "")
    ToolTip.visible: hovered || visualFocus
    ToolTip.delay: visualFocus ? 0 : 400
    contentItem: Item {
        implicitWidth: (iconImage.visible ? iconImage.width + (label.visible ? 8 : 0) : 0) + (label.visible ? label.implicitWidth : 0)
        implicitHeight: Math.max(iconImage.visible ? iconImage.height : 0,label.visible ? label.implicitHeight : 0)
        Image {
            id: iconImage; objectName: "buttonIcon"
            visible: control.asset.length > 0
            source: control.asset
            width: 20*control.theme.uiScale; height: 20*control.theme.uiScale
            anchors.left: parent.left; anchors.verticalCenter: parent.verticalCenter
            fillMode: Image.PreserveAspectFit
        }
        Text {
            id: label
            visible: !control.iconOnly || !control.asset
            anchors.left: iconImage.visible ? iconImage.right : parent.left
            anchors.leftMargin: iconImage.visible ? 8 : 0
            anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter
            text: control.text; elide: Text.ElideRight
            color: control.selected ? control.theme.selectedText : control.theme.foreground
            font.pixelSize: control.font.pixelSize
        }
    }
    background: Rectangle {
        radius: 6
        color: control.selected ? control.theme.selected : control.hovered || control.down ? control.theme.raised : "transparent"
        border.width: control.visualFocus ? 2 : control.selected ? 1 : 0
        border.color: control.visualFocus || control.selected ? control.theme.accent : control.theme.subtle
        Rectangle {
            visible: control.selected
            anchors.left: parent.left; anchors.verticalCenter: parent.verticalCenter
            width: 3; height: parent.height - 14; radius: 1
            color: control.theme.accent
        }
    }
    opacity: enabled ? 1 : 0.65
}
