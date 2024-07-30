<template>
  <div class="loader-outer m-auto">
    <svg class="loader-svg" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 60 60">
      <circle class="loader-circle text-red-900 trail-2" r="26" cx="30" cy="30" fill="transparent" />
      <circle class="loader-circle text-red-700 trail-1" r="26" cx="30" cy="30" fill="transparent" />
      <circle class="loader-circle text-red" r="26" cx="30" cy="30" fill="transparent" />
    </svg>
  </div>
</template>

<script setup lang="ts"></script>

<style>
.loader-outer {
  width: 60px;
  height: 60px;
}
.loader-svg {
  --inner-duration: 1.3s;
  --outer-duration: 2s;

  width: 100%;
  height: 100%;
  margin: auto;
  stroke-width: 6px;
  transform: rotate(0deg);
  animation: loading-animation var(--outer-duration) linear infinite;
}

@keyframes loading-animation {
  from {
    transform: rotate(0);
  }
  to {
    transform: rotate(360deg);
  }
}

.loader-circle {
  stroke: currentColor;
  animation:
    loading-animation-circle-array var(--inner-duration, 1s) cubic-bezier(0.215, 0.61, 0.355, 1) infinite,
    loading-animation-circle-offset var(--inner-duration, 1s) cubic-bezier(0.535, 0.655, 0.535, 1) infinite,
    loading-animation-opacity var(--inner-duration, 1s) linear infinite;
  stroke-dasharray: 163.36281798666926px;
  stroke-dashoffset: 163.36281798666926px;
  stroke-linecap: round;
}

.trail-1 {
  --full-opacity: 90%;
  animation-delay: 100ms;
}

.trail-2 {
  --full-opacity: 80%;
  animation-delay: 200ms;
}

@keyframes loading-animation-circle-array {
  0% {
    stroke-dasharray: 0, 200;
  }
  50% {
    stroke-dasharray: 100%, 200;
  }
  100% {
    stroke-dasharray: 0, 200;
  }
}
@keyframes loading-animation-circle-offset {
  0% {
    stroke-dashoffset: 0;
  }
  100% {
    stroke-dashoffset: -163.36px;
  }
}
@keyframes loading-animation-opacity {
  0% {
    opacity: 0;
  }
  5% {
    opacity: var(--full-opacity, 1);
  }
  75% {
    opacity: var(--full-opacity, 1);
  }
  90% {
    opacity: 0;
  }
  100% {
    opacity: 0;
  }
}
</style>
