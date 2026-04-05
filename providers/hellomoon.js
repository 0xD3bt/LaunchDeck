"use strict";

function getHelloMoonProvider() {
  return {
    id: "hellomoon",
    label: "Hello Moon QUIC",
    verified: true,
    supportsSingle: true,
    supportsSequential: true,
    supportsBundle: false,
  };
}

module.exports = {
  getHelloMoonProvider,
};
