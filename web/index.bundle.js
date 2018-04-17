/******/ (function(modules) { // webpackBootstrap
/******/ 	// The module cache
/******/ 	var installedModules = {};
/******/
/******/ 	// The require function
/******/ 	function __webpack_require__(moduleId) {
/******/
/******/ 		// Check if module is in cache
/******/ 		if(installedModules[moduleId]) {
/******/ 			return installedModules[moduleId].exports;
/******/ 		}
/******/ 		// Create a new module (and put it into the cache)
/******/ 		var module = installedModules[moduleId] = {
/******/ 			i: moduleId,
/******/ 			l: false,
/******/ 			exports: {}
/******/ 		};
/******/
/******/ 		// Execute the module function
/******/ 		modules[moduleId].call(module.exports, module, module.exports, __webpack_require__);
/******/
/******/ 		// Flag the module as loaded
/******/ 		module.l = true;
/******/
/******/ 		// Return the exports of the module
/******/ 		return module.exports;
/******/ 	}
/******/
/******/
/******/ 	// expose the modules object (__webpack_modules__)
/******/ 	__webpack_require__.m = modules;
/******/
/******/ 	// expose the module cache
/******/ 	__webpack_require__.c = installedModules;
/******/
/******/ 	// define getter function for harmony exports
/******/ 	__webpack_require__.d = function(exports, name, getter) {
/******/ 		if(!__webpack_require__.o(exports, name)) {
/******/ 			Object.defineProperty(exports, name, {
/******/ 				configurable: false,
/******/ 				enumerable: true,
/******/ 				get: getter
/******/ 			});
/******/ 		}
/******/ 	};
/******/
/******/ 	// define __esModule on exports
/******/ 	__webpack_require__.r = function(exports) {
/******/ 		Object.defineProperty(exports, '__esModule', { value: true });
/******/ 	};
/******/
/******/ 	// getDefaultExport function for compatibility with non-harmony modules
/******/ 	__webpack_require__.n = function(module) {
/******/ 		var getter = module && module.__esModule ?
/******/ 			function getDefault() { return module['default']; } :
/******/ 			function getModuleExports() { return module; };
/******/ 		__webpack_require__.d(getter, 'a', getter);
/******/ 		return getter;
/******/ 	};
/******/
/******/ 	// Object.prototype.hasOwnProperty.call
/******/ 	__webpack_require__.o = function(object, property) { return Object.prototype.hasOwnProperty.call(object, property); };
/******/
/******/ 	// __webpack_public_path__
/******/ 	__webpack_require__.p = "";
/******/
/******/
/******/ 	// Load entry module and return exports
/******/ 	return __webpack_require__(__webpack_require__.s = "./web/index.js");
/******/ })
/************************************************************************/
/******/ ({

/***/ "./web/audio.js":
/*!**********************!*\
  !*** ./web/audio.js ***!
  \**********************/
/*! no static exports found */
/***/ (function(module, exports) {

eval("const BEEP_FREQUENCY = 1000;\n\nclass Audio {\n  constructor(button) {\n    this.button = button\n    this.muted = true\n    this.context = new (window.AudioContext || window.webkitAudioContext)()\n    this.oscillator = this.context.createOscillator()\n\n    this.startOscillator()\n    this.button.onclick = (e) => { this.toggle(e.currentTarget) }\n  }\n\n  startOscillator() {\n    this.oscillator.type = 'sine'\n    this.oscillator.frequency.value = BEEP_FREQUENCY\n    this.oscillator.start()\n  }\n\n  mute() {\n    if (!this.muted) {\n      this.muted = true\n      this.oscillator.disconnect(this.context.destination)\n    }\n  }\n\n  unmute() {\n    if (this.muted) {\n      this.muted = false\n      this.oscillator.connect(this.context.destination)\n    }\n  }\n\n  toggle(button) {\n    let mutedIcon = button.querySelector('.muted')\n    let unmutedIcon = button.querySelector('.unmuted')\n    mutedIcon.hidden = !mutedIcon.hidden\n    unmutedIcon.hidden = !unmutedIcon.hidden\n    this.muted ? this.unmute() : this.mute()\n  }\n}\n\nmodule.exports = Audio;\n\n\n//# sourceURL=webpack:///./web/audio.js?");

/***/ }),

/***/ "./web/display.js":
/*!************************!*\
  !*** ./web/display.js ***!
  \************************/
/*! no static exports found */
/***/ (function(module, exports) {

eval("class Display {\n  constructor(canvas) {\n    this.canvas = canvas\n    this.context = canvas.getContext('2d')\n    this.numPixels = { x: 64, y: 32 }\n    this.pixelDimensions = { width: 10, height: 15 }\n\n    this.initContext()\n    this.initPixels()\n  }\n\n  initContext() {\n    this.context.webkitImageSmoothingEnabled = false\n    this.context.msImageSmoothingEnabled = false\n    this.context.imageSmoothingEnabled = false\n    this.context.fillRect(0, 0, this.canvas.width, this.canvas.height);\n  }\n\n  initPixels() {\n    this.pixels = new Array(this.numPixels.x * this.numPixels.y)\n    this.pixels.fill(false)\n  }\n\n  flipPixel(i) {\n    this.pixels[i] = !this.pixels[i]\n  }\n\n  drawPixel(i) {\n    let x = i % this.numPixels.x\n    let y = Math.floor(i / this.numPixels.x)\n    this.context.fillStyle = this.pixels[i] ? 'white' : 'black'\n    this.context.fillRect(\n      x * this.pixelDimensions.width,\n      y * this.pixelDimensions.height,\n      this.pixelDimensions.width,\n      this.pixelDimensions.height\n    )\n  }\n}\n\nmodule.exports = Display;\n\n\n//# sourceURL=webpack:///./web/display.js?");

/***/ }),

/***/ "./web/index.js":
/*!**********************!*\
  !*** ./web/index.js ***!
  \**********************/
/*! no static exports found */
/***/ (function(module, exports, __webpack_require__) {

eval("const Audio = __webpack_require__(/*! ./audio.js */ \"./web/audio.js\")\nconst Display = __webpack_require__(/*! ./display.js */ \"./web/display.js\")\nconst Keypad = __webpack_require__(/*! ./keypad.js */ \"./web/keypad.js\")\n\nconst canvas = document.getElementById('screen')\nconst mute = document.getElementById('mute')\n\nlet display = new Display(canvas)\nlet audio = new Audio(mute)\nlet keypad = new Keypad()\n\n\n//# sourceURL=webpack:///./web/index.js?");

/***/ }),

/***/ "./web/keypad.js":
/*!***********************!*\
  !*** ./web/keypad.js ***!
  \***********************/
/*! no static exports found */
/***/ (function(module, exports) {

eval("class Keypad {\n  constructor() {\n    this.initListeners()\n  }\n\n  initListeners() {\n    document.addEventListener('keydown', e => {\n      switch(e.key) {\n        case \"ArrowUp\":\n          console.log('UP')\n          break\n        case \"ArrowDown\":\n          console.log('DOWN')\n          break\n        case \"ArrowLeft\":\n          console.log('LEFT')\n          break\n        case \"ArrowRight\":\n          console.log('RIGHT')\n          break\n        case \"0\":\n        case \"1\":\n        case \"2\":\n        case \"3\":\n        case \"4\":\n        case \"5\":\n        case \"6\":\n        case \"7\":\n        case \"8\":\n        case \"9\":\n        case \"a\":\n        case \"b\":\n        case \"c\":\n        case \"d\":\n        case \"e\":\n        case \"f\":\n          console.log(e.key)\n          break\n        default:\n          break\n      }\n    })\n\n    document.addEventListener('keyup', e => {\n      switch(e.key) {\n        case \"ArrowUp\":\n          console.log('UP')\n          break\n        case \"ArrowDown\":\n          console.log('DOWN')\n          break\n        case \"ArrowLeft\":\n          console.log('LEFT')\n          break\n        case \"ArrowRight\":\n          console.log('RIGHT')\n          break\n        case \"0\":\n        case \"1\":\n        case \"2\":\n        case \"3\":\n        case \"4\":\n        case \"5\":\n        case \"6\":\n        case \"7\":\n        case \"8\":\n        case \"9\":\n        case \"a\":\n        case \"b\":\n        case \"c\":\n        case \"d\":\n        case \"e\":\n        case \"f\":\n          console.log(e.key)\n          break\n        default:\n          break\n      }\n    })\n  }\n}\n\nmodule.exports = Keypad;\n\n\n//# sourceURL=webpack:///./web/keypad.js?");

/***/ })

/******/ });