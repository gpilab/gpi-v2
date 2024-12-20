import { binaryOpInputs, PortDataType, singleInput, singleOutput } from "./portDefinition"
import { invoke } from "@tauri-apps/api"
import { GPI_Nodes } from "../../gpi"
import { Config, createNodeDef, NodeDefinition, NodeInputs, NodeOutputs } from "./nodeType"

//export const algebraNodes = ["_Add", "_Subtract", "_Multiply", "_Constant", "_pyAdd", "_DynamicNode"] as const

export const addNodeDefinition = createNodeDef({
  state: {
    type: "_Add",
    inputs: binaryOpInputs(["Real"]),
    output: singleOutput(["Real"]),
    config: {}
  },
  compute: ({ a, b }) => ["Integer", (a.value as unknown as number) + (b.value as unknown as number)]
})
//
//const subtractDef = createNodeDef({
//  state: {
//    ...addNodeDefinition.state,
//    type: "_Subtract",
//  },
//  compute: ({ a, b }) => a.value - b.value
//})
//
//const multiplyDef = createNodeDef({
//  state: {
//    ...addNodeDefinition.state,
//    type: "_Multiply",
//  },
//  compute: ({ a, b }) => a.value * b.value
//})
//
const constantDef = createNodeDef({
  state: {
    type: "_Constant",
    inputs: {},
    output: singleOutput(["Real"]),
    config: { value: 10 }
  },
  compute: (_, config) => ["Real", config.value]
})
//
//export const pyAddDef = createNodeDef({
//  state: {
//    type: "_pyAdd",
//    inputs: binaryOpInputs("Real"),
//    output: singleOutput("Real"),
//    config: { formula: "+ (py)" },
//  },
//  compute: async ({ a, b }) => {
//    const message = { "a": { "Real": a }, "b": { "Real": b } }
//    console.log("sending message to py_add:", message)
//    const val = await invoke<{ Real: number }>('py_add', message)
//    console.log("received from py_add:", val)
//    return val.Real
//  }
//})

export const arrayNodes = ["_Range", "_cos", "_sin", "_sinc", "_fft", "_Plot", "_Image", "_ArrayAdd", "_ArrayMult", "_PyAddArray"] as const


//export const rangeDef = createNodeDef({
//  state: {
//    type: "_Range",
//    inputs: {},
//    output: singleOutput("Vec1"),
//    config: {
//      start: -10,
//      end: 10,
//      step: .1
//    }
//  },
//  compute: (_, { start, end, step }) => {
//    console.log({ start, end, step })
//    return range(start, end, step)
//  }
//})
//
//export const sinDef = createNodeDef({
//  state: {
//    type: "_sin",
//    inputs: singleInput("Vec1"),
//    output: singleOutput("Vec1"),
//    config: { amplitude: 1, phaseOffset: 0, frequency: 4 }
//  },
//  compute: ({ a }, { amplitude, phaseOffset, frequency }) =>
//    a.value.map(e =>
//      amplitude * Math.sin(e * frequency + phaseOffset)
//    )
//})
//
//export const cosDef = createNodeDef({
//  state: {
//    ...sinDef.state,
//    type: "_cos",
//  },
//  compute: ({ a }, { amplitude, phaseOffset, frequency }) =>
//    a.value.map(e =>
//      amplitude * Math.cos(e * frequency + phaseOffset)
//    )
//})
//
//export const sincDef = createNodeDef({
//  state: {
//    ...sinDef.state,
//    type: "_sinc",
//  },
//  compute: ({ a }, { amplitude, phaseOffset, frequency }) =>
//    a.value.map(e => {
//      const x = e * frequency + phaseOffset
//
//      if (x == 0) {
//        return amplitude * 1
//      }
//
//      return amplitude * Math.sin(PI * x) / (PI * x)
//    }
//    )
//})
//
//export const arrayAddDef = createNodeDef({
//  state: {
//    type: "_ArrayAdd",
//    inputs: binaryOpInputs("Vec1"),
//    output: singleOutput("Vec1"),
//    config: { formula: "\\textbf{+}" }
//  },
//  compute: ({ a, b }) => a.value.map((e, i) => e + b.value[i])
//})
//
//export const arrayMultiplyDef = createNodeDef({
//  state: {
//    type: "_ArrayMult",
//    inputs: binaryOpInputs("Vec1"),
//    output: singleOutput("Vec1"),
//    config: { formula: "\\times" }
//  },
//  compute: ({ a, b }) => a.value.map((e, i) => e * b.value[i])
//})
//
//export const fftDef = createNodeDef({
//  state: {
//    type: "_fft",
//    inputs: singleInput("Vec1"),
//    output: singleOutput("Vec1"),
//    config: { formula: "\\mathcal{F}\\{f(x)\\}" },
//  },
//  compute: async ({ a }) => {
//    const val = await invoke<number[]>('fft', { signal: a })
//    console.log(Math.max(...val))
//    return val
//  }
//})
//
//export const pyAddArrayDef = createNodeDef({
//  state: {
//    type: "_PyAddArray",
//    inputs: binaryOpInputs("Vec1"),
//    output: singleOutput("Vec1"),
//    config: { formula: "+ (py array)" },
//  },
//  compute: async ({ a, b }) => {
//    const message = {
//      message: {
//        node_type: 'add_int_array',
//        inputs: [{ "Vec": a }, { "Vec": b }]
//      }
//    }
//    console.log("sending message to run_node:", message)
//    const val = await invoke<{ "Vec": number[] }>('run_node', message)
//    console.log(" received from run_node:", message)
//    return val.Vec
//  }
//})
//
export const plotDef = createNodeDef({
  state: {
    type: "_Plot",
    inputs: singleInput(["Array", ["Real"]]),
    output: singleOutput(["Array", ["Real"]]),
    config: {}
  },
  compute: ({ a }) => a.value
})
export const imageDef = createNodeDef({
  state: {
    type: "_Image",
    inputs: singleInput(["Image"]),
    output: singleOutput(["Image"]),
    config: {}
  },
  compute: ({ a }) => a.value
})




export const createDynamicNode = (type: string, config: Config,
  inputs: NodeInputs,
  output: NodeOutputs) => {
  //console.log("creating node:", { type, inputs, output, config })

  return createNodeDef({
    state: {
      type: type,
      inputs,
      output,
      config
    },
    compute: async (input_values) => {

      //format inputs as {"name:{"data_type":value}} 
      //const input_formatted =
      //  Object.entries(input_values)
      //    .reduce<Record<string, PortDataType>>((acc, [_, port]) => {
      //      const t = port.data[0]
      //      const v = port.dataType[1]
      //      return {
      //        ...acc,
      //        port.dataType[0]: port.dataType[1]
      //      }
      //    })
      const formatted_input = Object.entries(input_values).reduce<Record<string, Record<string, any>>>((acc, [name, port]) => ({ ...acc, [name]: { [port.value[0]]: port.value[1] } }), {})
      const dynamic_message = {
        message: {
          node_type: type,
          inputs: formatted_input
        }
      }
      //console.log("using python node!:", dynamic_message)
      /// list of input `Values` to pass to `node_type`'s python "compute" function
      const val = await invoke<Record<"out", PortDataType>>('run_node', dynamic_message)
      //console.log("dynamic node value", val)
      const out = val["out"]
      const key = Object.keys(out)[0]
      // @ts-ignore
      const portValue = [key, out[key]] as PortDataType
      //console.log("dynamic node value", portValue)
      return portValue
    }

  })
}


export const defaultDynamicNodeDef = createNodeDef({
  state: {
    type: "_DynamicNode",
    inputs: binaryOpInputs(["Real"]),
    output: singleOutput(["Real"]),
    config: {}
  },
  compute: async ({ a, b }) => {
    const dynamic_message = {
      message: {
        node_type: "add_int",
        inputs: [["Integer", a], ["Integer", b]]
      }
    }
    console.log("using dynamic node!:", dynamic_message)
    /// list of input `Values` to pass to `node_type`'s python "compute" function
    const val = await invoke<Record<"out", PortDataType>>('run_node', dynamic_message)
    console.log("dynamic node value", val)
    const out = val["out"]
    const key = Object.keys(out)[0]
    // @ts-ignore
    const portValue = [key, out[key]] as PortDataType
    console.log("dynamic node value", portValue)
    return portValue
  }
})

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const nodeDefaultDefinitions = {
  //"_Add": addNodeDefinition,
  //"_Subtract": subtractDef,
  //"_Multiply": multiplyDef,
  "_Constant": constantDef,
  //"_Range": rangeDef,
  //"_sin": sinDef,
  //"_cos": cosDef,
  //"_sinc": sincDef,
  //"_fft": fftDef,
  //"_ArrayAdd": arrayAddDef,
  //"_ArrayMult": arrayMultiplyDef,
  "_Plot": plotDef,
  "_Image": imageDef,
  //"_pyAdd": pyAddDef,
  //"_PyAddArray": pyAddArrayDef,
  "_DynamicNode": defaultDynamicNodeDef,
}

export const getDefaultNodeDefinition = (nodeType: string) => {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  // TODO: clean this up
  // @ts-ignore
  const built_in = nodeDefaultDefinitions[nodeType] as NodeDefinition<any, any, any> | undefined
  if (built_in) {
    console.log("using builtin node", built_in)
    return built_in
  }
  const node = GPI_Nodes.find(n => n.state.type == nodeType)
  if (node === undefined) {
    console.log(`tried to get node definition for ${nodeType},but it is not in the known list of nodes`, GPI_Nodes)
    throw "Node not found!" + nodeType
  }
  console.log("using dynamic node from python", node)
  return node
}

