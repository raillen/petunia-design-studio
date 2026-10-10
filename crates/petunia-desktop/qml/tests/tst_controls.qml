import QtQuick
import QtQuick.Controls
import QtTest
import ".."

TestCase {
    id: testCase
    name: "StudioControls"
    when: windowShown
    width: 600; height: 400; visible: true
    StudioTheme { id: tokens }
    StudioButton { id: button; theme: tokens; text: "Selecionar"; KeyNavigation.tab: numeric; x: 20; y: 20 }
    NumericField { id: numeric; theme: tokens; number: 12; fieldName: "X"; onCommitted: value => numeric.number = value; KeyNavigation.backtab: button; x: 20; y: 80 }
    SignalSpy { id: committed; target: numeric; signalName: "committed" }
    function typeText(value) { for (var i=0;i<value.length;i++) keyClick(value[i]) }
    function init() { numeric.number = 12; numeric.draftDirty = false; numeric.text = "12"; committed.clear() }
    function test_theme_independence() {
        tokens.appearance = "highContrast"
        compare(String(tokens.accent), "#ffff00")
        tokens.density = "compact"
        compare(tokens.target,32)
        compare(String(tokens.accent), "#ffff00")
        tokens.appearance = "dark"; tokens.density = "comfortable"
    }
    function test_button_keyboard_focus() {
        button.forceActiveFocus()
        verify(button.activeFocus)
        keyClick(Qt.Key_Tab)
        verify(numeric.activeFocus)
    }
    function test_numeric_keyboard_units() {
        committed.clear(); numeric.text = "12"; numeric.forceActiveFocus()
        keyClick(Qt.Key_Up)
        compare(committed.count,1)
        compare(committed.signalArguments[0][0],13)
        keyClick(Qt.Key_Down, Qt.ShiftModifier)
        compare(committed.signalArguments[1][0],3)
    }
    function test_untouched_precision() {
        numeric.number = 12.123456
        numeric.forceActiveFocus()
        compare(numeric.text,"12.123456")
        button.forceActiveFocus()
        compare(committed.count,0)
        compare(numeric.number,12.123456)
    }
    function test_edited_blur_once() {
        numeric.forceActiveFocus(); numeric.selectAll(); typeText("23.987654")
        button.forceActiveFocus()
        compare(committed.count,1)
        compare(numeric.number,23.987654)
    }
    function test_cancel_then_blur() {
        numeric.forceActiveFocus(); numeric.selectAll(); typeText("93")
        keyClick(Qt.Key_Escape); button.forceActiveFocus()
        compare(committed.count,0)
        compare(numeric.number,12)
    }
    function test_numeric_cancel() {
        numeric.text = "99"; numeric.forceActiveFocus(); keyClick(Qt.Key_Escape)
        compare(Number(numeric.text),12)
    }
}
