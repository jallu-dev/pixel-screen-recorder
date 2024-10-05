import React from "react";

const SpeedEdit = ({ onSpeedClick, speedRate }) => {
  return (
    <div className="w-full flex gap-2 px-3 mb-4 items-center">
      <p className="uppercase text-zinc-200 w-28">Speed</p>
      <button
        id="2"
        onClick={(e) => onSpeedClick(e)}
        className={`${
          speedRate == "2" ? "bg-indigo-600" : "bg-zinc-600"
        } hover:bg-opacity-80 text-zinc-100 px-4 py-2 mx-1 rounded-md uppercase`}
      >
        0.5X
      </button>
      <button
        id="1"
        onClick={(e) => onSpeedClick(e)}
        className={`${
          speedRate == "1" ? "bg-indigo-600" : "bg-zinc-600"
        } hover:bg-opacity-80 text-zinc-100 px-4 py-2 mx-1 rounded-md uppercase`}
      >
        1X
      </button>
      <button
        id="0.5"
        onClick={(e) => onSpeedClick(e)}
        className={`${
          speedRate == "0.5" ? "bg-indigo-600" : "bg-zinc-600"
        } hover:bg-opacity-80 text-zinc-100 px-4 py-2 mx-1 rounded-md uppercase`}
      >
        2X
      </button>
      <button
        id="0.25"
        onClick={(e) => onSpeedClick(e)}
        className={`${
          speedRate == "0.25" ? "bg-indigo-600" : "bg-zinc-600"
        } hover:bg-opacity-80 text-zinc-100 px-4 py-2 mx-1 rounded-md uppercase`}
      >
        4X
      </button>
      <button
        id="0.1"
        onClick={(e) => onSpeedClick(e)}
        className={`${
          speedRate == "0.1" ? "bg-indigo-600" : "bg-zinc-600"
        } hover:bg-opacity-80 text-zinc-100 px-4 py-2 mx-1 rounded-md uppercase`}
      >
        10X
      </button>
    </div>
  );
};

export default SpeedEdit;
