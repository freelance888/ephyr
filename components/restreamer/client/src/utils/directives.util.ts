export type SaveOrCloseHandlers = {
  save: () => void;
  close: () => void;
};

export const saveOrCloseByKeys = (
  node: HTMLElement,
  params: SaveOrCloseHandlers,
) => {
  const keyPressEventName = 'keypress';
  const keyDownEventName = 'keydown';

  const handleKeyPress = (event) => {
    if (event.key === 'Enter' && params.save) {
      params.save();
    }
  };

  const handleKeyDown = (event) => {
    if (event.key === 'Escape' && params.close) {
      params.close();
    }
  };

  node.addEventListener(keyPressEventName, handleKeyPress);
  node.addEventListener(keyDownEventName, handleKeyDown);

  if (node?.tabIndex < 0) {
    node.tabIndex = -1;
  }

  setTimeout(() => {
    node.focus();
  });

  return {
    destroy() {
      node.removeEventListener(keyPressEventName, handleKeyPress);
      node.removeEventListener(keyDownEventName, handleKeyDown);
    },
  };
};
