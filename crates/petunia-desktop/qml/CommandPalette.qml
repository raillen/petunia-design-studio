pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Dialog {
    id: dialog
    required property StudioTheme theme
    required property var studioState
    signal activated(string actionId)
    property var results: (studioState.actions || []).filter(function(action) { return action.label.toLowerCase().indexOf(search.text.toLowerCase()) >= 0 })
    title: "Paleta de comandos"
    modal: true; width: Math.min(620*theme.uiScale,parent ? parent.width-32 : 620)
    x: parent ? (parent.width-width)/2 : 0; y: parent ? Math.max(16,(parent.height-height)/3) : 0
    onOpened: { search.text = ""; search.forceActiveFocus(); resultsView.currentIndex = 0 }
    function activate(index) {
        if (index >= 0 && index < results.length && results[index].enabled !== false) { var id = results[index].id; close(); activated(id) }
    }
    contentItem: ColumnLayout {
        TextField {
            id: search; objectName: "commandSearch"; Layout.fillWidth: true
            placeholderText: "Buscar ação…"; Accessible.name: "Buscar comando"
            onTextChanged: resultsView.currentIndex = 0
            Keys.onDownPressed: resultsView.currentIndex = Math.min(dialog.results.length-1,resultsView.currentIndex+1)
            Keys.onUpPressed: resultsView.currentIndex = Math.max(0,resultsView.currentIndex-1)
            Keys.onReturnPressed: dialog.activate(resultsView.currentIndex)
        }
        ListView {
            id: resultsView; Layout.fillWidth: true; Layout.preferredHeight: Math.min(340,Math.max(80,contentHeight))
            clip: true; model: dialog.results; spacing: 4
            delegate: StudioButton {
                required property var modelData
                required property int index
                width: resultsView.width
                theme: dialog.theme
                text: modelData.label + (modelData.shortcut ? "    " + modelData.shortcut : "")
                explanation: modelData.enabled === false ? modelData.reason || "Ação indisponível no contexto atual" : ""
                selectable: true; selected: resultsView.currentIndex === index
                onClicked: { resultsView.currentIndex = index; dialog.activate(index) }
            }
            ScrollBar.vertical: ScrollBar {}
        }
        Label { visible: !dialog.results.length; text: "Nenhum comando encontrado."; color: dialog.theme.secondary }
        Label { text: "↑ ↓ navegar · Enter executar · Escape fechar"; color: dialog.theme.secondary }
    }
}
