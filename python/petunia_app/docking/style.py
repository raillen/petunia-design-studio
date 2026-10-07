"""Dark theme stylesheet matching Affinity Designer 2 for PySide6QtAds and QMainWindow."""

ADS_DARK_THEME = """
/* ========================================================================= */
/* Main Window and Menus                                                     */
/* ========================================================================= */
QMainWindow {
    background-color: #18181a;
    color: #f2f2f5;
}

QMenuBar {
    background-color: #18181a;
    color: #a1a1aa;
    border-bottom: 1px solid #27272a;
    font-size: 11px;
    padding: 2px 4px;
}

QMenuBar::item {
    background: transparent;
    padding: 4px 8px;
    border-radius: 3px;
}

QMenuBar::item:selected, QMenuBar::item:pressed {
    background-color: #2e2e33;
    color: #ffffff;
}

QMenu {
    background-color: #1f1f21;
    color: #f2f2f5;
    border: 1px solid #333336;
    padding: 4px;
    font-size: 11px;
}

QMenu::item {
    padding: 5px 24px 5px 20px;
    border-radius: 3px;
}

QMenu::item:selected {
    background-color: #2e2e33;
    color: #ffffff;
}

QMenu::item:disabled {
    color: #52525b;
}

QMenu::separator {
    height: 1px;
    background-color: #2e2e33;
    margin: 4px 6px;
}

/* ========================================================================= */
/* PySide6QtAds: CDockContainerWidget & Splitter                             */
/* ========================================================================= */
ads--CDockContainerWidget {
    background-color: #18181a;
    border: none;
}

ads--CDockSplitter {
    background-color: #181819;
    width: 2px;
    height: 2px;
}

ads--CDockSplitter::handle {
    background-color: #1c1c1e;
}

ads--CDockSplitter::handle:hover {
    background-color: #38383c;
}

/* ========================================================================= */
/* PySide6QtAds: CDockAreaWidget & TitleBar                                  */
/* ========================================================================= */
ads--CDockAreaWidget {
    background-color: #28282a;
    border: 1px solid #181819;
}

ads--CDockAreaTitleBar {
    background-color: #1c1c1e;
    border-bottom: 1px solid #181819;
    min-height: 25px;
    max-height: 25px;
    padding: 0px 2px;
}

/* ========================================================================= */
/* PySide6QtAds: CDockWidgetTab                                              */
/* ========================================================================= */
ads--CDockWidgetTab {
    background-color: #1c1c1e;
    color: #8e8e93;
    border: none;
    border-right: 1px solid #242426;
    padding: 3px 6px;
    font-size: 10px;
    font-weight: normal;
}

ads--CDockWidgetTab:hover {
    background-color: #242426;
    color: #e4e4e7;
}

ads--CDockWidgetTab[activeTab="true"] {
    background-color: #28282a;
    color: #f2f2f5;
    border-right: 1px solid #242426;
    border-bottom: 1px solid #28282a;
    font-weight: 600;
}

/* Close button inside tab */
ads--CDockWidgetTab QToolButton,
ads--CDockWidgetTab ads--CTitleBarButton {
    background: transparent;
    border: none;
    padding: 0px;
    margin-left: 4px;
}

/* ========================================================================= */
/* PySide6QtAds: TitleBar Action Buttons                                     */
/* ========================================================================= */
ads--CDockAreaTitleBar QToolButton,
ads--CTitleBarButton {
    background: transparent;
    border: none;
    padding: 2px;
    color: #a1a1a6;
    border-radius: 2px;
    qproperty-iconSize: 12px 12px;
}

ads--CDockAreaTitleBar QToolButton:hover,
ads--CTitleBarButton:hover {
    background-color: #38383c;
    color: #ffffff;
}

ads--CDockAreaTitleBar QToolButton:pressed,
ads--CTitleBarButton:pressed {
    background-color: #444448;
}

/* ========================================================================= */
/* PySide6QtAds: Floating and AutoHide                                       */
/* ========================================================================= */
ads--CFloatingDockContainer {
    border: 1px solid #38383c;
    background-color: #18181a;
}

ads--CAutoHideSideBar {
    background-color: #18181a;
    border-right: 1px solid #27272a;
}

ads--CAutoHideTab {
    background-color: #1f1f20;
    color: #a1a1a6;
    border: 1px solid #27272a;
    padding: 4px 8px;
    font-size: 10px;
}

ads--CAutoHideTab:hover {
    background-color: #28282b;
    color: #ffffff;
}

ads--CAutoHideTab[activeTab="true"] {
    background-color: #28282a;
    color: #f2f2f5;
    border-left: 2px solid #38bdf8;
}
"""
