<script setup lang="ts">
import mini from '@/assets/boards-live/mini.png'
import standard from '@/assets/boards-live/standard.png'
import large from '@/assets/boards-live/shuuro-live.png'
import { BoardSize, type Description } from '@/helpers/variantDescription'
import { onUnmounted, ref } from 'vue'

const props = defineProps<{
  container: HTMLElement
  variants: Description[]
  name: BoardSize
  currentVariant: Description | undefined
}>()
const showImg = ref(false)
const showVariants = ref(false)
const height = ref(0)

setTimeout(() => {
  showImg.value = true
  setTimeout(() => {
    showVariants.value = true
  }, 1500)
}, 100)

defineEmits(['back', 'viewVariant'])

function isSelected(v: Description) {
  return (
    v.variant == props.currentVariant?.variant && v.subVariant == props.currentVariant.subVariant
  )
}

function variantName() {
  if (props.name == BoardSize.Mini) {
    return 'mini'
  } else if (props.name == BoardSize.Standard) {
    return 'standard'
  } else if (props.name == BoardSize.Large) {
    return 'large'
  }
}

function img() {
  if (props.name == BoardSize.Mini) return mini
  else if (props.name == BoardSize.Standard) return standard
  else if (props.name == BoardSize.Large) return large
}

function resize() {
  height.value = props.container.getBoundingClientRect().height
}

function imgStyle() {
  let size = height.value * (0.7 - (showVariants.value == true ? 0.2 : 0))
  return `width: ${size}px; height: ${size}px`
}

window.addEventListener('resize', resize)
resize()

onUnmounted(() => {
  window.removeEventListener('resize', resize)
})
</script>

<template>
  <div
    class="text-main-900 dark:text-main-200 text-right font-bold text-2xl font-lichess flex justify-end items-center"
  >
    <span
      @click="$emit('back')"
      class="p-3 dark:border-main-500 cursor-pointer transition-[scale] duration-350 hover:scale-120"
      ></span
    >
  </div>

  <h1
    class="-mt-5 text-center font-sec text-main-900 dark:text-main-200 font-bold capitalize md:text-2xl lg:text-3xl"
  >
    {{ variantName() }} chess
  </h1>

  <div class="p-2 flex flex-col gap-5 justify-around font-sec dark:text-main-100/80 text-center">
    <Transition @after-leave="showVariants = true" name="variant-img">
      <img
        @click="showVariants = true"
        v-if="showImg"
        :src="img()"
        :style="imgStyle()"
        class="mx-auto transition-[width_height]"
      />
    </Transition>
    <div class="mx-auto">
      <div v-if="showVariants" class="flex gap-6 flex-wrap justify-around font-main">
        <button
          v-for="variant in props.variants"
          class="appearance-none hover:cursor-pointer basis-[33%] lg:basis-auto p-2 lg:p-4 bg-gradient-to-bl shadow shadow-main-100 dark:shadow-main-700 transition-[box-shadow] duration-300 hover:shadow-main-800 hover:dark:shadow-main-200 rounded-md border-l-1 md:text-xl lg:text-2xl"
          :class="{
            ' from-main-200 to-main-300 dark:from-main-700 dark:to-main-900 ': !isSelected(variant),
            ' from-main-300 to-main-400 dark:from-main-800 dark:to-main-950 border-l-4 underline ':
              isSelected(variant),
          }"
          @click="$emit('viewVariant', variant)"
        >
          {{ variant.name }}
        </button>
      </div>
    </div>
  </div>
</template>

<style>
.variant-img-enter-active,
.variant-img-leave-active {
  transition:
    opacity 0.3s ease,
    transform 0.5s ease-in,
    filter 0.5s ease-in;
}

.variant-img-enter-from {
  opacity: 80%;
  transform: scale(0.7);
  filter: contrast(200%) blur(9px);
}

.variant-img-enter-to {
  opacity: 100%;
  transform: scale(1);
  filter: contrast(100%);
}

.variant-img-leave-from {
  transform: translateX(0px);
  opacity: 100%;
}
.variant-img-leave-to {
  transform: translateX(-100px);
  opacity: 0%;
}
</style>
