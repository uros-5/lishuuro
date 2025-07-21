<script setup lang="ts">
import type { Description } from '@/helpers/variantDescription'

const props = defineProps<{ variant: Description }>()

defineEmits(['back', 'selectVariant'])
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
  <Transition name="load-variant" :appear="true">
    <div
      class="-mt-5 text-center font-sec text-main-900 dark:text-main-200 font-bold capitalize md:text-2xl lg:text-3xl flex flex-col gap-5"
    >
      <h1 class="text-main-900 dark:text-main-100">{{ props.variant.name }}</h1>
      <div class="text-center font-sec capitalize md:text-xl lg:text-2xl">
        <p v-if="props.variant.points > 0">{{ props.variant.points }} points</p>
      </div>
      <div class="font-sec capitalize md:text-xl lg:text-2xl">
        <span class="relative">
          <Transition
            @after-enter="
              (e) => {
                ;(e as HTMLElement).style.width = '100%'
              }
            "
            name="hidden-option"
            :appear="true"
          >
            <div
              v-if="!props.variant.selection"
              class="absolute left-[0%] top-[50%] h-[10%] bg-red-500"
            ></div>
          </Transition>

          selection
        </span>
      </div>

      <div class="font-sec capitalize md:text-xl lg:text-2xl">
        <span class="relative">
          <Transition
            @after-enter="
              (e) => {
                ;(e as HTMLElement).style.width = '100%'
              }
            "
            name="hidden-option"
            :appear="true"
          >
            <div
              v-if="!props.variant.placement"
              class="absolute left-[0%] top-[50%] h-[10%] bg-red-500"
            ></div>
          </Transition>

          placement
        </span>
      </div>

      <div class="font-sec lowercase md:text-xl lg:text-2xl" v-if="props.variant.fairyPieces">
        <span class="relative"> with Fairy pieces </span>
      </div>
      <div
        class="font-sec lowercase md:text-xl lg:text-2xl flex flex-col gap-5 justify-center items-center"
      >
        <div class="mx-auto w-[50%]" v-for="picture in props.variant.picture">
          <img class="" :src="picture" />
        </div>
      </div>

      <div class="flex justify-center mb-3">
        <button
          @click="$emit('selectVariant', props.variant)"
          class="font-sec text-md mx-auto border-2 rounded border-main-900 dark:border-main-100/20 dark:bg-main-900/50 px-5 rotate-[0.3deg] cursor-pointer shadow shadow-main-200 dark:shadow-main-700 transition-[background] duration-200 hover:bg-main-200 hover:dark:bg-main-700"
        >
          Choose this variant
        </button>
      </div>
    </div>
  </Transition>
</template>

<style>
.hidden-option-enter-active {
  transition: width 3s ease-in;
}

.hidden-option-enter-from {
  width: 0%;
}

.hidden-option-enter-to {
  width: 100%;
}

.load-variant-enter-active {
  transition:
    opacity 1s ease-in,
    transform 1.2s ease;
}

.load-variant-enter-from {
  opacity: 0;
  transform: scale(0%);
}

.load-variant-enter-to {
  opacity: 1;
  transform: scale(100%);
}
</style>
