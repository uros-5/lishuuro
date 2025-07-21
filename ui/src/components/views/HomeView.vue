<script setup lang="ts">
import { onMounted, onUnmounted, useTemplateRef } from 'vue'
import { RouterLink } from 'vue-router'
import { variants, type Description } from '@/helpers/variantDescription'
import { useWs } from '@/stores/ws'
import { ev2, rmEv2, type Event2 } from '@/helpers/customEvent'
import VariantsPicker from '../VariantsPicker.vue'
import VariantPicker from '../VariantPicker.vue'
import { allowedDuration, updateState } from '@/helpers/home_state'
import { defineAsyncComponent } from 'vue'
import { useHome } from '@/stores/home'
import router from '@/router'
import { backend, newTitle } from '@/helpers/backend'
import {
  MessageType,
  type GameRequest,
  type GamesCount,
  type PlayersCount,
  type RedirectPlayer,
  type TypeOfGame,
} from '@/helpers/rust_types'

const CreateGameModal = defineAsyncComponent({
  loader: () => import('../CreateGameModal.vue'),
  timeout: 300,
})

const home = useHome()

const variantsContainer = useTemplateRef('variant-container')

const msg = new Map<MessageType, any>()
msg.set(MessageType.PlayerCount, (data: PlayersCount) => {
  home.players = data.count
})
msg.set(MessageType.GameCount, (data: GamesCount) => {
  home.games = data.count
})
msg.set(MessageType.RedirectToGame, (data: RedirectPlayer) => {
  ws.redirectToastOpen = true
  router.push(`/game/${data.game}`)
})

function onMessage(event: Event2) {
  const fn = msg.get(event.detail.t)
  if (fn) {
    fn(event.detail)
  }
}

function selectVariant(variant: Description) {
  home.selectedVariant = variant
  updateState(home.minute, home.increment, home.selectedVariant, home.selectedCategory)
  setTimeout(() => {
    home.currentView = ''
    home.viewVariant = undefined
    home.animateSelected = true
    setTimeout(() => {
      home.animateSelected = false
    }, 1000)
  }, 500)
}

function sendGameRequest(newColor: number) {
  function gameType(): TypeOfGame | undefined {
    if (home.aiLevel.selected) {
      return {
        content: home.aiLevel.level,
        type: 'VsAi',
      }
    } else if (home.friend.selected) {
      return { type: 'VsFriend', content: home.friend.name }
    }
  }
  home.color = newColor
  if (home.selectedVariant == undefined) {
    return
  }
  let game_type = gameType()
  if (game_type == undefined) return
  let request = {
    t: MessageType.AddGameRequest,
    d: {
      minutes: allowedDuration[home.minute - 1],
      incr: allowedDuration[home.increment - 1],
      variant: home.selectedVariant.variant,
      sub_variant: home.selectedVariant.subVariant,
      color: home.color,
      game_type: game_type,
    },
  }
  ws.SEND(request)
  home.aiLevel.level = 0
  home.aiLevel.selected = false
  home.friend.name = ''
  home.friend.selected = false
}

const ws = useWs()
let listener

onMounted(() => {
  listener = ev2('wsMessage', onMessage)
  document.body.dataset['page'] = 'home'
  ws.SEND({ t: MessageType.ChangeRoom, d: 'home' })
  newTitle('Home')
})

onUnmounted(() => {
  rmEv2(listener!)
  ws.SEND({ t: MessageType.ChangeRoom, d: '' })
})
</script>

<template>
  <section
    id="home"
    class="col-start-3 col-end-4 row-span-1 grid grid-cols-1 sm:grid-cols-[0.05fr_0.9fr_0.05fr] md:grid-cols-[0.05fr_0.3fr_0.8fr_0.05fr] py-5 gap-x-2 gap-y-7 touch-auto"
    :class="{
      ' grid-rows-[15vh_50vh] md:grid-rows-[20vh_auto] ': true,
      ' grid-rows-[15vh_auto] md:grid-rows-[10vh_auto] ': false,
    }"
  >
    <div class="invisible hidden md:block row-span-1"></div>
    <div
      class="row-start-1 row-end-1 md:row-start-2 md:row-end-2 col-start-1 col-end-2 sm:col-start-2 md:col-end-3 text-main-950 dark:text-main-100 flex flex-col justify-center items-center gap-5 md:gap-8 border-0 md:text-2xl"
    >
      <div class="md:hidden"></div>
      <div class="hidden md:block font-main">
        <RouterLink class="text-4xl mt-3" to="/">lishuuro.org</RouterLink>
      </div>
      <RouterLink
        class="border-b-3 border-main-300 dark:border-main-500 shadow shadow-main-200/10 px-1"
        to="/how-to-play-shuuro"
        >Shuuro rules</RouterLink
      >

      <a
        v-if="ws.reg == false"
        class="bg-main-300/50 dark:bg-main-800/50 rounded-md shadow shadow-main-200 dark:shadow-main-700 hover:bg-main-300 hover:dark:bg-main-900 px-3 font-sec"
        :href="backend() + '/login'"
        >Login</a
      >
      <RouterLink
        v-else-if="ws.reg == true"
        class="bg-main-300/50 dark:bg-main-800/50 rounded-md shadow shadow-main-200 dark:shadow-main-700 hover:bg-main-300 hover:dark:bg-main-900 px-3 font-sec"
        :to="`/@/${ws.username}`"
        >@</RouterLink
      >
    </div>

    <div
      ref="variant-container"
      class="row-start-2 row-end-2 md:row-start-2 md:row-end-3 col-start-1 col-end-2 sm:col-start-2 md:col-start-3 md:col-end-3 shadow shadow-main-300 dark:shadow-main-700 dark:text-main-50 flex relative overflow-x-hidden bg-linear-140 dark:bg-linear-150 from-main-200/80 from-20% dark:from-0% dark:from-main-900 to-main-50/80 dark:to-main-950 bg-main-300/50 dark:bg-main-700/50"
    >
      <Transition name="slide-up">
        <div
          v-if="home.currentView == 'options'"
          class="absolute w-full h-full bg-linear-140 dark:bg-linear-150 from-main-200 from-20% dark:from-0% dark:from-main-900 to-main-100 dark:to-main-950 bg-main-300/50 dark:bg-main-700/50 shadow shadow-main-300 dark:shadow-main-700 z-10 overflow-y-scroll"
        >
          <VariantsPicker
            :container="variantsContainer!"
            :name="home.selectedCategory"
            :variants="variants.filter((item) => item.category == home.selectedCategory)"
            :current-variant="home.selectedVariant"
            @back="((home.currentView = ''), (home.viewVariant = undefined))"
            @view-variant="
              (variant: Description) => {
                home.viewVariant = variant
              }
            "
          />
        </div>
      </Transition>

      <Transition name="slide-right">
        <div
          v-if="home.currentView == 'options' && home.viewVariant != undefined"
          class="absolute w-full h-full bg-linear-140 dark:bg-linear-150 from-main-200 from-20% dark:from-0% dark:from-main-900 to-main-100 dark:to-main-950 bg-main-300/50 dark:bg-main-700/50 shadow shadow-main-300 dark:shadow-main-700 z-20 overflow-y-scroll"
        >
          <VariantPicker
            @back="((home.currentView = 'options'), (home.viewVariant = undefined))"
            @select-variant="(variant) => selectVariant(variant)"
            :variant="home.viewVariant"
          />
        </div>
      </Transition>

      <div class="w-full flex flex-row row-span-1 md:col-start-3 md:col-end-4 md:text-2xl">
        <div
          class="rotate-[0.3deg] basis-[100%] grid grid-cols-[0.25fr_2fr_0.25fr] md:grid-cols-[0.4fr_0.2fr_2fr_0.2fr_0.4fr] lg:grid-cols-[0.8fr_0.2fr_2fr_0.2fr_0.8fr] xl:grid-cols-[0.6fr_0.2fr_2fr_0.2fr_0.6fr] gap-2 md:gap-7 p-2 font-sec"
        >
          <div class="row-span-1 col-span-5 invisible h-[1vh]"></div>
          <p
            id="minute"
            class="row-span-1 col-start-1 col-end-3 md:col-start-2 md:col-end-4 select-none"
          >
            Minute: <span>{{ allowedDuration[home.minute - 1] }}</span>
          </p>
          <input
            v-model.number="home.minute"
            @change="
              updateState(home.minute, home.increment, home.selectedVariant, home.selectedCategory)
            "
            class="slider row-span-1 col-start-2 col-end-2 md:col-start-2 md:col-end-5 rounded h-4 cursor-pointer"
            type="range"
            min="1"
            max="28"
            id="minuteSlider"
          />
          <p
            id="increment"
            class="row-span-1 col-start-1 col-end-3 md:col-start-2 md:col-end-4 select-none"
          >
            Increment: <span>{{ allowedDuration[home.increment - 1] }}</span>
          </p>
          <input
            v-model.number="home.increment"
            @change="
              updateState(home.minute, home.increment, home.selectedVariant, home.selectedCategory)
            "
            class="slider row-span-1 col-start-2 col-end-2 md:col-start-2 md:col-end-5 cursor-pointer"
            type="range"
            min="1"
            max="27"
            id="incrementSlider"
          />

          <div class="text-center row-span-1 col-start-2 col-end-2 md:col-start-2 md:col-end-5">
            Variant:
          </div>

          <div
            class="flex justify-center gap-3 md:gap-5 lg:gap-7 min-h-0 md:h-auto row-span-1 col-start-2 col-end-2 md:col-start-2 md:col-end-5"
          >
            <button
              v-for="(category, i) in ['mini', 'standard', 'large']"
              @click="((home.currentView = 'options'), (home.selectedCategory = i))"
              class="bg-main-200 dark:bg-main-900/50 w-[50%] px-2 rotate-[0.3deg] duration-200 shadow shadow-main-100/15 cursor-pointer text-center rounded capitalize transition-transform"
              :class="{
                ' border-0 border-t-2 border-main-800 dark:border-main-300/90 ':
                  home.selectedCategory != i,
                ' border-2 border-main-700 dark:border-main-200/90 ': home.selectedCategory == i,
                ' selected-category ': home.animateSelected && home.selectedCategory == i,
              }"
            >
              {{ category }}
            </button>
          </div>
          <div
            class="appearance-none row-span-1 col-start-1 col-end-4 md:col-start-2 md:col-end-5 flex justify-between gap-5 md:gap-7 text-center"
          >
            <button
              to="/game/someid"
              @click="home.aiModal = !home.aiModal"
              class="rounded border md:gap-5 lg:gap-7-main-900 dark:border-main-100/20 dark:bg-main-900/50 md:h-auto w-[50%] px-5 rotate-[0.3deg] cursor-pointer triangle shadow shadow-main-200 transition-[background] duration-200 dark:shadow-main-700 hover:bg-main-300 hover:dark:bg-main-700"
            >
              Play vs AI
            </button>
            <button
              @click="home.friendModal = !home.friendModal"
              class="border-2 rounded border-main-900 dark:border-main-100/20 dark:bg-main-900/50 md:h-auto w-[50%] px-5 rotate-[0.3deg] cursor-pointer shadow shadow-main-200 dark:shadow-main-700 transition-[background] duration-200 hover:bg-main-300 hover:dark:bg-main-700"
            >
              Play vs friend
            </button>
          </div>
          <div class="row-span-1 col-span-5 invisible h-[1vh]"></div>
        </div>
      </div>
    </div>

    <div
      class="row-span-1 md:row-start-3 md:row-end-3 md:col-start-3 md:col-end-4 col-start-1 col-end-2 sm:col-start-2 flex flex-col justify-center items-center text-main-950 dark:text-main-100 font-sec md:text-xl"
    >
      <RouterLink to="/tv"
        >Current games: <span>{{ home.games }}</span></RouterLink
      >
      <div>
        Active players: <span>{{ home.players }}</span>
      </div>
    </div>

    <div
      class="row-span-1 md:row-start-4 md:row-end-4 col-start-1 col-end-2 sm:col-start-2 md:col-start-3 md:col-end-4 flex justify-around gap-5 items-center text-main-950 dark:text-main-100 font-sec md:text-lg"
    >
      <a
        href="https://github.com/uros-5/lishuuro"
        class="shadow shadow-main-200 dark:shadow-main-700 px-3"
        >Improve servers and source code</a
      >
    </div>

    <CreateGameModal
      @save-form="
        (request, myColor) => {
          home.friend = request
          sendGameRequest(myColor)
        }
      "
      request-type="vsFriend"
      v-if="home.friendModal"
      @closeModal="home.friendModal = false"
    />
    <CreateGameModal
      @save-form="
        (e, myColor) => {
          home.aiLevel = e
          sendGameRequest(myColor)
        }
      "
      request-type="vsAi"
      v-if="home.aiModal"
      @closeModal="home.aiModal = false"
    />
  </section>
</template>

<style>
.slide-up-enter-active {
  transition: all 0.5s ease-in;
}

.slide-up-leave-active {
  transition: all 0.7s ease-out;
}

.slide-up-enter-from {
  border-top: 5px solid black;
  transform: translate(0%, 101%);
}

.slide-up-enter-to {
  transform: translate(0%, 0%);
}

.slide-up-leave-from {
  border-top: 5px solid black;
  transform: translate(0%, 0%);
}

@media (prefers-color-scheme: dark) {
  .slide-up-leave-from {
    border-top: 5px solid white;
    transform: translate(0%, 0%);
  }
}

.slide-up-leave-to {
  transform: translate(0%, 100%);
}

.slide-right-enter-active {
  transition: all 0.5s ease-in;
}

.slide-right-leave-active {
  transition: all 0.7s ease-out;
}

.slide-right-enter-from {
  border-left: 5px solid black;
  transform: translate(101%, 0%);
}

.slide-right-enter-to {
  transform: translate(0%, 0%);
}

.slide-right-leave-from {
  border-left: 5px solid black;
  transform: translate(0%, 0%);
}

@media (prefers-color-scheme: dark) {
  .slide-right-leave-from {
    border-left: 5px solid white;
    transform: translate(0%, 0%);
  }
}

.slide-right-leave-to {
  transform: translate(100%, 0%);
}

@keyframes notify {
  0% {
    transform: scale(100%);
  }

  25% {
    transform: scale(80%);
  }

  75% {
    transform: scale(90%);
  }

  100% {
    transform: scale(100%);
  }
}

.selected-category {
  animation: 0.5s infinite alternate notify;
}
</style>
