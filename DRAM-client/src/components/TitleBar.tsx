import { Window } from "@tauri-apps/api/window";
import fullscreenIcon from "../assets/icons/fullscreenbtnicon.svg";
import closeIcon from "../assets/icons/closeappbtnicon.svg";

type TitleBarProps = {
  showMaximize?: boolean;
};

function TitleBar({ showMaximize = false }: TitleBarProps) {
  const handleMinimize = () => Window.getCurrent().minimize();
  const handleMaximize = () => Window.getCurrent().toggleMaximize();
  const handleClose = () => Window.getCurrent().close();

  return (
    <div className="titlebar" data-tauri-drag-region>
      <button
        className="titlebar-btn titlebar-minimize"
        onMouseDown={(e) => e.stopPropagation()}
        onClick={handleMinimize}
        type="button"
      >
        —
      </button>
      {showMaximize && (
        <button
          className="titlebar-btn titlebar-maximize"
          onMouseDown={(e) => e.stopPropagation()}
          onClick={handleMaximize}
          type="button"
        >
          <img src={fullscreenIcon} width="12" height="12" />
        </button>
      )}
      <button
        className="titlebar-btn titlebar-close"
        onMouseDown={(e) => e.stopPropagation()}
        onClick={handleClose}
        type="button"
      >
        <img src={closeIcon} width="14" height="14" />
      </button>
    </div>
  );
}

export default TitleBar;