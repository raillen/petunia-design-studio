pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls

TextField {
    id: field
    required property StudioTheme theme
    property real number: 0
    property string fieldName: ""
    property real minimum: -1000000
    property real maximum: 1000000
    property bool draftDirty: false
    property string originalText: ""
    signal committed(real value)
    signal cancelled()
    font.pixelSize: 14*theme.uiScale
    color: theme.foreground
    Binding {
        target: field; property: "text"; value: String(field.number)
        when: !field.draftDirty
        restoreMode: Binding.RestoreNone
    }
    onActiveFocusChanged: if (activeFocus) { originalText = String(number); draftDirty = false }
    onTextEdited: draftDirty = true
    selectByMouse: true
    implicitHeight: theme.target
    implicitWidth: 92
    Accessible.name: fieldName
    validator: DoubleValidator { bottom: field.minimum; top: field.maximum; decimals: 12; locale: "C" }
    function submit() {
        if (!draftDirty) return
        var value = Number(text)
        if (isFinite(value) && value >= minimum && value <= maximum && acceptableInput && value !== number) committed(value)
        draftDirty = false
        text = String(number)
    }
    onEditingFinished: submit()
    Keys.onUpPressed: event => {
        var value = Number(text)
        if (isFinite(value)) { draftDirty = true; text = String(Math.min(maximum, value + (event.modifiers & Qt.ShiftModifier ? 10 : 1))); submit() }
    }
    Keys.onDownPressed: event => {
        var value = Number(text)
        if (isFinite(value)) { draftDirty = true; text = String(Math.max(minimum, value - (event.modifiers & Qt.ShiftModifier ? 10 : 1))); submit() }
    }
    Keys.onEscapePressed: { draftDirty = false; text = originalText || String(number); cancelled() }
    background: Rectangle {
        color: field.theme.raised; radius: 4
        border.width: field.activeFocus ? 2 : 1
        border.color: field.activeFocus ? field.theme.accent : field.theme.controlBorder
    }
}
