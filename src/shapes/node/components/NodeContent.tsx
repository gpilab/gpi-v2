import { TLShapeId, track, useEditor } from "tldraw";
import { NodeShape, NodeShapeProps } from "../nodeShapeUtil";
import { ConstantNode } from "./constantNode";
import { PlotNode } from "./plotNode";
import { RangeNode } from "./rangeNode";
import { useTheme } from "../../../util/useTheme";
import { createContext, useContext } from "react";
import { TrigNode } from "./trigNode";
import { BlockMath } from "react-katex";
import { ImageNode } from "./imageNode";


export type NodeConfig = {
  id: TLShapeId,
} & NodeShapeProps


export const NodeConfigContext = createContext<NodeConfig | null>(null)

export const useNodeConfig = () => {
  const context = useContext(NodeConfigContext)
  if (context === null) {
    throw Error("useNodeConfig must be called from inside NodeConfigContext.Provider")
  }
  return context
}
export const NodeContent = track((props: { nodeShape: NodeShape }) => {
  const { nodeShape } = props
  const { nodeType } = nodeShape.props
  const editor = useEditor()
  const nodeConfig = {
    id: nodeShape.id,
    ...nodeShape.props
  }


  const updateNode = (updatedProps: Partial<NodeShapeProps>) => {
    console.log("updating node with props...", updatedProps)
    editor.updateShape({
      id: nodeShape.id,
      type: "node",
      props: { ...nodeShape.props, ...updatedProps }
    })
  }

  return <NodeConfigContext.Provider value={nodeConfig}>
    {(() => {
      switch (nodeType) {
        case ("_Constant"):
          return <ConstantNode updateNode={updateNode} />
        case ("_Plot"):
          return <PlotNode />
        case ("_Image"):
          return <ImageNode />
        case ("_Range"):
          return <RangeNode updateNode={updateNode} />
        case ("_sin"):
        case ("_sinc"):
        case ("_cos"):
          return <TrigNode updateNode={updateNode} />
        default:
          return <DefaultNode />
      }
    })()}
  </NodeConfigContext.Provider >
})


const DefaultNode = track(() => {
  const { nodeType, config } = useNodeConfig()
  const { formula } = config
  const theme = useTheme()
  const infoColor = theme.grey

  return <div style={{ width: "100%", height: "100%", }}>
    {formula !== undefined
      ? <div style={{ height: "100%", display: "flex", justifyContent: "center", alignItems: "center" }}>
        <BlockMath math={formula} />
      </div>
      :
      <div style={{
        color: infoColor, padding: "5px",
        display: "flex", justifyContent: "end"
      }}>
        {nodeType}
      </div>
    }
  </div>
})

