import React from "react";

const TrimEdit = ({ onTrimTimeChanged, enforceMinMax, trimTime, duration }) => {
  return (
    <div className="w-full flex gap-2 px-3 mb-4 items-center">
      <p className="uppercase text-zinc-200 w-28">Trim</p>
      <div className="flex flex-col gap-2">
        <div className="border border-zinc-400 text-zinc-200 px-4 py-2 mx-1 rounded-md uppercase flex items-center">
          <p className="uppercase text-zinc-200 w-24">start time</p>
          <input
            type="number"
            step={1}
            className="rounded-md w-14 bg-transparent border border-zinc-400 focus:border-indigo-600 outline-non focus:outline-none py-1 px-3 "
            placeholder="Hr"
            id="startHr"
            min={0}
            max={trimTime.endHr}
            onChange={(e) => {
              onTrimTimeChanged(e);
            }}
            onBlur={(e) => {
              enforceMinMax(e);
              onTrimTimeChanged(e);
            }}
            value={trimTime.startHr}
          />
          <p className="px-2">:</p>
          <input
            type="number"
            step={1}
            className="rounded-md w-14 bg-transparent border border-zinc-400 focus:border-indigo-600 outline-non focus:outline-none py-1 px-3 "
            placeholder="Min"
            id="startMin"
            min={0}
            max={trimTime.startHr < trimTime.endHr ? 59 : trimTime.endMin}
            onChange={(e) => {
              onTrimTimeChanged(e);
            }}
            onBlur={(e) => {
              enforceMinMax(e);
              onTrimTimeChanged(e);
            }}
            value={trimTime.startMin}
          />
          <p className="px-2">:</p>
          <input
            type="number"
            step={1}
            className="rounded-md w-14 bg-transparent border border-zinc-400 focus:border-indigo-600 outline-non focus:outline-none py-1 px-3 "
            placeholder="Sec"
            id="startSec"
            min={0}
            max={
              trimTime.startHr < trimTime.endHr
                ? 59
                : trimTime.startMin < trimTime.endMin
                ? 59
                : trimTime.endSec
            }
            onChange={(e) => {
              onTrimTimeChanged(e);
            }}
            onBlur={(e) => {
              enforceMinMax(e);
              onTrimTimeChanged(e);
            }}
            value={trimTime.startSec}
          />
        </div>
        <div className="border border-zinc-400 text-zinc-200 px-4 py-2 mx-1 rounded-md uppercase flex items-center">
          <p className="uppercase text-zinc-200 w-24">end time</p>
          <input
            type="number"
            step={1}
            className="rounded-md w-14 bg-transparent border border-zinc-400 focus:border-indigo-600 outline-non focus:outline-none py-1 px-3 "
            placeholder="Hr"
            id="endHr"
            min={trimTime.startHr}
            max={duration.hr}
            onChange={(e) => {
              onTrimTimeChanged(e);
            }}
            onBlur={(e) => {
              enforceMinMax(e);
              onTrimTimeChanged(e);
            }}
            value={trimTime.endHr}
          />
          <p className="px-2">:</p>
          <input
            type="number"
            step={1}
            className="rounded-md w-14 bg-transparent border border-zinc-400 focus:border-indigo-600 outline-non focus:outline-none py-1 px-3 "
            placeholder="Min"
            id="endMin"
            min={trimTime.startHr < trimTime.endHr ? 0 : trimTime.startMin}
            max={trimTime.startHr < trimTime.endHr ? 59 : duration.min}
            onChange={(e) => {
              onTrimTimeChanged(e);
            }}
            onBlur={(e) => {
              enforceMinMax(e);
              onTrimTimeChanged(e);
            }}
            value={trimTime.endMin}
          />
          <p className="px-2">:</p>
          <input
            type="number"
            step={1}
            className="rounded-md w-14 bg-transparent border border-zinc-400 focus:border-indigo-600 outline-non focus:outline-none py-1 px-3 "
            placeholder="Sec"
            id="endSec"
            min={
              trimTime.startHr < trimTime.endHr
                ? 0
                : trimTime.startMin < trimTime.endMin
                ? 0
                : trimTime.startSec
            }
            max={
              trimTime.startHr < trimTime.endHr
                ? 59
                : trimTime.startMin < trimTime.endMin
                ? 59
                : duration.sec
            }
            onChange={(e) => {
              onTrimTimeChanged(e);
            }}
            onBlur={(e) => {
              enforceMinMax(e);
              onTrimTimeChanged(e);
            }}
            value={trimTime.endSec}
          />
        </div>
      </div>
    </div>
  );
};

export default TrimEdit;
