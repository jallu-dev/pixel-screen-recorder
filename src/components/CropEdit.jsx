import React from "react";

const CropEdit = ({
  fakeCropArea,
  realCropArea,
  screenSize,
  onRealValuesChange,
  enforceMinMax,
  ratio,
}) => {
  return (
    <div className="w-full flex gap-2 px-3 mb-4 items-center">
      <p className="uppercase text-zinc-200 w-28">Crop</p>
      <div className="border border-zinc-400 p-2 mx-1 rounded-md flex">
        <div
          className={`h-[100px] bg-black mx-1 relative`}
          style={{ width: `${ratio * 100}px` }}
        >
          <div
            className="border-4 border-indigo-600 absolute"
            style={{
              top: `${fakeCropArea.y}px`,
              left: `${fakeCropArea.x}px`,
              width: `${fakeCropArea.width}px`,
              height: `${fakeCropArea.height}px`,
            }}
          ></div>
        </div>
        <div className="flex flex-col ml-3">
          <div className="flex mb-3 mr-8 items-center">
            <p className="uppercase text-zinc-200 w-6">X</p>
            <input
              type="number"
              step={1}
              className="rounded-md w-16 bg-transparent text-zinc-200 border border-zinc-400 focus:border-indigo-600 outline-non focus:outline-none py-1 px-3"
              id="x"
              onChange={(e) => {
                enforceMinMax(e);
                onRealValuesChange(e);
              }}
              min={0}
              max={screenSize.width - realCropArea.width}
              value={realCropArea.x}
            />
          </div>
          <div className="flex mr-8 items-center">
            <p className="uppercase text-zinc-200 w-6">Y</p>
            <input
              type="number"
              step={1}
              className="rounded-md w-16 bg-transparent text-zinc-200 border border-zinc-400 focus:border-indigo-600 outline-non focus:outline-none py-1 px-3"
              id="y"
              onChange={(e) => {
                enforceMinMax(e);
                onRealValuesChange(e);
              }}
              min={0}
              max={screenSize.height - realCropArea.height}
              value={realCropArea.y}
            />
          </div>
        </div>
        <div className="flex flex-col">
          <div className="flex mb-3 items-center">
            <p className="uppercase text-zinc-200 w-16">Width</p>
            <input
              type="number"
              step={1}
              className="rounded-md w-16 bg-transparent text-zinc-200 border border-zinc-400 focus:border-indigo-600 outline-non focus:outline-none py-1 px-3"
              id="width"
              onChange={(e) => {
                enforceMinMax(e);
                onRealValuesChange(e);
              }}
              min={0}
              max={screenSize.width - realCropArea.x}
              value={realCropArea.width}
            />
          </div>
          <div className="flex items-center">
            <p className="uppercase text-zinc-200 w-16">Height</p>
            <input
              type="number"
              step={1}
              className="rounded-md w-16 bg-transparent text-zinc-200 border border-zinc-400 focus:border-indigo-600 outline-non focus:outline-none py-1 px-3"
              id="height"
              onChange={(e) => {
                enforceMinMax(e);
                onRealValuesChange(e);
              }}
              min={0}
              max={screenSize.height - realCropArea.y}
              value={realCropArea.height}
            />
          </div>
        </div>
      </div>
    </div>
  );
};

export default CropEdit;
