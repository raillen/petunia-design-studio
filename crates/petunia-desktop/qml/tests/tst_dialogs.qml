import QtQuick
import QtQuick.Controls
import QtTest
import ".."

TestCase {
    id: testCase
    name: "StudioDialogs"
    visible: true; width: 900; height: 700
    when: windowShown
    StudioTheme { id: tokens }
    property var state: ({
        preferences:{appearance:"dark",iconFamily:"phosphor",iconStyle:"outline",density:"comfortable",uiScale:1,reducedMotion:false},
        actions:[{id:"tool.select",label:"Selecionar",shortcut:"V",enabled:true},{id:"tool.pen",label:"Caneta",shortcut:"P",enabled:true}],error:""
    })
    QtObject {
        id: fakeBackend
        property string stateJson: "{}"
        property var calls: []
        function iconSource(key) { return "" }
        function invoke(command,payload) { calls = calls.concat([{command:command,payload:JSON.parse(payload)}]); return true }
    }
    PreferencesDialog { id: preferencesDialog; theme: tokens; backend: fakeBackend; studioState: testCase.state }
    ShortcutDialog { id: shortcuts; theme: tokens; backend: fakeBackend; studioState: testCase.state }
    CommandPalette { id: commands; theme: tokens; studioState: testCase.state }
    SignalSpy { id: activated; target: commands; signalName: "activated" }
    function init() { fakeBackend.calls = []; activated.clear() }
    function cleanup() { preferencesDialog.close(); shortcuts.close(); commands.close() }
    function test_preferences_cancel_preserves_state() {
        preferencesDialog.open(); tryCompare(preferencesDialog,"opened",true)
        findChild(preferencesDialog,"iconFamilyChoice").currentIndex = 1
        keyClick(Qt.Key_Escape)
        tryCompare(preferencesDialog,"visible",false)
        compare(fakeBackend.calls.length,0)
        compare(state.preferences.iconFamily,"phosphor")
    }
    function test_preferences_apply_scale() {
        preferencesDialog.open(); tryCompare(preferencesDialog,"opened",true)
        findChild(preferencesDialog,"interfaceScale").currentIndex = 4
        mouseClick(preferencesDialog.footer.standardButton(Dialog.Apply))
        compare(fakeBackend.calls.length,1)
        compare(fakeBackend.calls[0].command,"preferences")
        compare(fakeBackend.calls[0].payload.uiScale,2)
        tryCompare(preferencesDialog,"visible",false)
    }
    function test_shortcut_conflict_before_apply() {
        shortcuts.open(); tryCompare(shortcuts,"opened",true)
        shortcuts.actionId = "tool.pen"
        findChild(shortcuts,"shortcutEditor").text = "V"
        compare(shortcuts.conflict,"Selecionar")
        compare(fakeBackend.calls.length,0)
    }
    function test_palette_keyboard_action() {
        commands.open(); tryCompare(commands,"opened",true)
        var search = findChild(commands,"commandSearch")
        verify(search.activeFocus)
        keyClick(Qt.Key_Down); keyClick(Qt.Key_Return)
        compare(activated.count,1)
        compare(activated.signalArguments[0][0],"tool.pen")
        tryCompare(commands,"visible",false)
    }
}
