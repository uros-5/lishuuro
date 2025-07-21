<script setup lang="ts">
import { ntw } from '@/not-tailwind'
import { onMounted, ref, watch, type Ref } from 'vue'
import selectionImg from '../../assets/boards-live/selection-extended.png'
import placement1 from '../../assets/boards-live/selection-2.png'
import placement2 from '../../assets/boards-live/howto2.png'
import live from '../../assets/boards-live/howto3.png'
import meme from '../../assets/boards-live/intro-meme.jpg'
import { newTitle } from '@/helpers/backend'

const question = ref(false)
const learning = ref(false)
const selectionElem = ref<HTMLElement>()
const placementElem = ref<HTMLElement>()
const playElem = ref<HTMLElement>()

onMounted(() => {
  document.body.dataset['page'] = 'rules'
  newTitle('How to play shuuro ?')
})
function yes() {
  question.value = true
  setTimeout(() => {
    learning.value = true
  }, 600)
}

;[selectionElem, placementElem, playElem].forEach((item) => {
  watch(item, () => {
    setTimeout(() => {
      item.value?.classList.remove(ntw.get('text-transparent'))
      item.value?.classList.add(ntw.get('dark:text-main-200'))
      item.value?.classList.add(ntw.get('text-main-800'))
    }, 400)
  })
})
</script>

<template>
  <div
    id="rules"
    class="col-start-3 col-end-4 row-span-1 grid gap-y-6 grid-cols-[0.2fr_0.6fr_0.2fr] lg:grid-cols-[0.4fr_0.8fr_0.4fr] grid-rows-[var(--empty-space,30vh)_auto_auto] font-main text-main-900 dark:text-main-200 relative"
    :class="{ ' [--empty-space:15vh] md:[--empty-space:10vh] gap-y-3 ': learning == true }"
  >
    <div class="row-span-1 col-span-3"></div>
    <Transition name="first-question">
      <div
        v-if="!question"
        class="row-span-1 col-start-2 col-end-3 text-center text-xl sm:text-2xl md:text-3xl"
      >
        Do you know how to play chess?
      </div>
    </Transition>
    <Transition name="first-question">
      <div
        v-if="!question"
        class="row-span-1 col-span-3 text-2xl flex justify-around sm:justify-center sm:gap-9"
      >
        <button
          @click="yes()"
          class="border shadow-main-500 cursor-pointer rounded before:text-xl before:sm:text-2xl before:lg:text-3xl before:text-green-800 dark:before:text-green-400 before:align-middle font-lichess before:font-normal before:normal-case before:opacity-90 bg-gradient-to-b hover:from-main-400 hover:dark:from-main-950 dark:from-main-800 from-main-200 dark:to-main-900 to-main-100 before:content-[attr(data-icon)] before:p-8"
          data-icon=""
        ></button>

        <a
          href="https://lichess.org/learn"
          target="_blank"
          class="border shadow-main-500 cursor-pointer rounded before:text-xl before:sm:text-2xl before:lg:text-3xl before:text-red-500 dark:before:text-red-300 before:align-middle font-lichess before:font-normal before:normal-case before:opacity-90 bg-gradient-to-b hover:from-main-400 hover:dark:from-main-950 dark:from-main-800 from-main-200 dark:to-main-900 to-main-100 before:content-[attr(data-icon)] before:p-8"
          data-icon=""
        ></a></div
    ></Transition>
    <Transition name="learning">
      <!-- <div v-if="learning == true && question == false"> -->
      <div
        class="row-span-1 col-start-2 col-end-3 text-2xl flex flex-col items-center justify-center"
        v-if="learning"
      >
        <h1
          ref="selectionElem"
          class="text-3xl inline-block bg-gradient-to-r bg-clip-text from-main-300 via-main-100 to-main-800 text-transparent transition-[color] duration-200"
        >
          1. Selection
        </h1>
      </div>
    </Transition>
    <div class="row-span-1 col-span-3 md:flex md:justify-center animate-[1s_img]" v-if="learning">
      <div src="" class="w-[80%] h-[80%] md:w-auto md:h-auto mx-auto my-2">
        <img :src="selectionImg" />
      </div>
    </div>

    <Transition name="learning" style="--tr1: 2.2s" appear>
      <!-- <div v-if="learning == true && question == false"> -->
      <div
        class="row-span-1 col-start-2 col-end-3 text-2xl flex flex-col items-center justify-center"
        v-if="learning"
      >
        <h1
          ref="placementElem"
          class="text-3xl inline-block bg-gradient-to-r bg-clip-text from-main-300 via-main-100 to-main-800 text-transparent transition-[color] duration-200"
        >
          2. Placement
        </h1>
      </div>
    </Transition>

    <div class="row-span-1 col-span-3 md:flex md:justify-center animate-[1s_img]" v-if="learning">
      <div src="" class="w-[80%] h-[80%] md:w-auto md:h-auto mx-auto my-2">
        <img :src="placement1" />
      </div>
    </div>

    <div class="row-span-1 col-span-3 md:flex md:justify-center animate-[1s_img]" v-if="learning">
      <div src="" class="w-[80%] h-[80%] md:w-auto md:h-auto mx-auto my-2">
        <img :src="placement2" />
      </div>
    </div>

    <Transition name="learning" style="--tr1: 2.5s" appear>
      <!-- <div v-if="learning == true && question == false"> -->
      <div
        class="row-span-1 col-start-2 col-end-3 text-2xl flex flex-col items-center justify-center"
        v-if="learning"
      >
        <h1
          ref="playElem"
          class="text-3xl inline-block bg-gradient-to-r bg-clip-text from-main-300 via-main-100 to-main-800 text-transparent transition-[color] duration-200"
        >
          3. Play
        </h1>
      </div>
    </Transition>

    <div class="row-span-1 col-span-3 md:flex md:justify-center animate-[1s_img]" v-if="learning">
      <div src="" class="w-[80%] h-[80%] md:w-auto md:h-auto mx-auto my-2">
        <img :src="live" />
      </div>
    </div>
    <div
      v-if="learning"
      class="row-span-1 col-span-3 md:col-start-2 md:col-end-3 p-4 text-justify md:text-center md:text-xl lg:text-2xl"
    >
      <b class="font-lichess text-main-950 dark:text-main-100">*</b> only knights can jump on these
      blocks, they can capture other pieces as well
    </div>

    <div
      class="row-span-1 col-span-3 md:col-start-2 md:col-end-3 md:flex md:justify-center animate-[1s_img]"
      v-if="learning"
    >
      <div src="" class="mx-auto my-2">
        <img class="w-[80%] h-[80%] md:w-auto md:h-auto mx-auto" :src="meme" />
      </div>
    </div>
  </div>
</template>

<style>
.first-question-enter-active,
.first-question-leave-active {
  transition:
    opacity 0.7s ease,
    scale 0.6s ease-in-out;
}

.first-question-enter-from,
.first-question-leave-to {
  opacity: 0;
  scale: 0.85;
}

.learning-enter-active,
.learning-leave-active {
  transition:
    opacity 1.7s ease,
    transform var(--tr1, 0.8s) ease-in-out,
    color 1s ease-in-out;
}

.learning-enter-from,
.learning-leave-to {
  opacity: 0;
  transform: translateX(-70px);
}

@keyframes img {
  from {
    opacity: 0;
    transform: translateX(30vw);
    scale: 95%;
  }

  to {
    transform: translateX(0px);
    scale: 100%;
    opacity: 1;
  }
}
</style>
